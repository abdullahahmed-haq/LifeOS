use std::{
    fs::OpenOptions,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use lifeos_domain::{
    AREA_TYPE_ID, ActionReceipt, AppError, AppSettings, Area, AreaVersion, DEFAULT_DEVICE_ID,
    DEFAULT_USER_ID, DEFAULT_WORKSPACE_ID, EntityRevision, HealthSnapshot, SearchResult,
    ThemePreference, UndoResult,
};
use lifeos_search::{fts_query, normalize_for_search};
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, TransactionBehavior, backup::Backup, params,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const INITIAL_SCHEMA: &str = r#"
CREATE TABLE workspaces (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE users (id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id), display_name TEXT NOT NULL, locale TEXT NOT NULL, timezone TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
CREATE TABLE devices (id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id), label TEXT NOT NULL, platform TEXT NOT NULL, install_id TEXT NOT NULL UNIQUE, created_at INTEGER NOT NULL, last_seen_at INTEGER NOT NULL);
CREATE TABLE entity_type_definitions (id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id), key TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('core','custom')), label_en TEXT NOT NULL, label_ar TEXT NOT NULL, fallback_label TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, UNIQUE(workspace_id,key));
CREATE TABLE entities (id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id), type_id TEXT NOT NULL REFERENCES entity_type_definitions(id), title TEXT NOT NULL, description TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, archived_at INTEGER, deleted_at INTEGER, revision INTEGER NOT NULL CHECK(revision >= 1), created_by_type TEXT NOT NULL, created_by_id TEXT, updated_by_type TEXT NOT NULL, updated_by_id TEXT, origin_device_id TEXT REFERENCES devices(id));
CREATE INDEX entities_active_idx ON entities(workspace_id,type_id,deleted_at,archived_at,updated_at DESC);
CREATE TABLE areas (entity_id TEXT PRIMARY KEY REFERENCES entities(id), status TEXT NOT NULL, sort_rank REAL NOT NULL DEFAULT 0);
CREATE TABLE domain_events (sequence INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT NOT NULL UNIQUE, workspace_id TEXT NOT NULL REFERENCES workspaces(id), event_type TEXT NOT NULL, aggregate_entity_id TEXT, aggregate_revision INTEGER, payload_json TEXT NOT NULL, actor_type TEXT NOT NULL, actor_id TEXT, operation_id TEXT NOT NULL, occurred_at INTEGER NOT NULL);
CREATE TABLE undo_batches (id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id), operation_id TEXT NOT NULL, status TEXT NOT NULL CHECK(status IN ('available','undone')), created_at INTEGER NOT NULL, undone_at INTEGER);
CREATE TABLE undo_operations (id TEXT PRIMARY KEY, undo_batch_id TEXT NOT NULL REFERENCES undo_batches(id), ordinal INTEGER NOT NULL, action_key TEXT NOT NULL, payload_json TEXT NOT NULL, UNIQUE(undo_batch_id,ordinal));
CREATE TABLE audit_events (id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspaces(id), action_key TEXT NOT NULL, actor_type TEXT NOT NULL, actor_id TEXT, affected_entities_json TEXT NOT NULL, operation_id TEXT NOT NULL, undo_batch_id TEXT REFERENCES undo_batches(id), occurred_at INTEGER NOT NULL);
CREATE TABLE entity_versions (id TEXT PRIMARY KEY, entity_id TEXT NOT NULL REFERENCES entities(id), revision INTEGER NOT NULL, snapshot_json TEXT NOT NULL, changed_fields_json TEXT NOT NULL, operation_id TEXT NOT NULL, created_at INTEGER NOT NULL, UNIQUE(entity_id,revision));
CREATE TABLE backup_metadata (id TEXT PRIMARY KEY, created_at INTEGER NOT NULL, schema_version INTEGER NOT NULL, integrity_result TEXT NOT NULL);
CREATE VIRTUAL TABLE entity_search USING fts5(entity_id UNINDEXED, workspace_id UNINDEXED, type_key UNINDEXED, title, normalized_text, tokenize='unicode61');
INSERT INTO workspaces VALUES ('00000000-0000-7000-8000-000000000001','LifeOS',0,0);
INSERT INTO users VALUES ('00000000-0000-7000-8000-000000000002','00000000-0000-7000-8000-000000000001','Local User','en','Africa/Cairo',0,0);
INSERT INTO devices VALUES ('00000000-0000-7000-8000-000000000003','00000000-0000-7000-8000-000000000001','This device','desktop','local-install',0,0);
INSERT INTO entity_type_definitions VALUES ('00000000-0000-7000-8000-000000000004','00000000-0000-7000-8000-000000000001','area','core','Area','مجال','Area',0,0);
"#;

const FOUNDATION_SETTINGS_SCHEMA: &str = r#"
CREATE TABLE app_settings (
  workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id),
  locale TEXT NOT NULL CHECK(locale IN ('en','ar')),
  theme TEXT NOT NULL CHECK(theme IN ('light','dark','system')),
  timezone TEXT NOT NULL,
  week_starts_on INTEGER NOT NULL CHECK(week_starts_on BETWEEN 0 AND 6),
  revision INTEGER NOT NULL CHECK(revision >= 1),
  updated_at INTEGER NOT NULL
);
INSERT INTO app_settings VALUES ('00000000-0000-7000-8000-000000000001','en','system','Africa/Cairo',1,1,0);
CREATE TABLE permission_policies (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  subject_kind TEXT NOT NULL,
  operation TEXT NOT NULL,
  decision TEXT NOT NULL CHECK(decision IN ('allow','ask','deny')),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(workspace_id, subject_kind, operation)
);
CREATE TABLE background_jobs (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  kind TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('queued','running','succeeded','failed','cancelled')),
  payload_json TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0 CHECK(attempts >= 0),
  available_at INTEGER NOT NULL,
  started_at INTEGER,
  completed_at INTEGER,
  last_error_code TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX background_jobs_ready_idx ON background_jobs(workspace_id,state,available_at);
CREATE TABLE consumer_cursors (
  consumer_key TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  last_domain_event_sequence INTEGER NOT NULL DEFAULT 0 CHECK(last_domain_event_sequence >= 0),
  updated_at INTEGER NOT NULL
);
"#;

const MIGRATIONS: &[(i32, &str)] = &[(1, INITIAL_SCHEMA), (2, FOUNDATION_SETTINGS_SCHEMA)];

pub struct EntityStore {
    connection: Connection,
}

impl EntityStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let mut connection = Connection::open(path).map_err(internal)?;
        connection
            .busy_timeout(Duration::from_millis(2_000))
            .map_err(internal)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(internal)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(internal)?;
        connection
            .pragma_update(None, "synchronous", "NORMAL")
            .map_err(internal)?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn health(&self) -> Result<HealthSnapshot, AppError> {
        Ok(HealthSnapshot {
            schema_version: self
                .connection
                .query_row("PRAGMA user_version", [], |r| r.get(0))
                .map_err(internal)?,
            sqlite_version: self
                .connection
                .query_row("SELECT sqlite_version()", [], |r| r.get(0))
                .map_err(internal)?,
            journal_mode: self
                .connection
                .query_row("PRAGMA journal_mode", [], |r| r.get(0))
                .map_err(internal)?,
            foreign_keys_enabled: self
                .connection
                .query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i32>(0))
                .map_err(internal)?
                == 1,
        })
    }

    pub fn app_settings(&self) -> Result<AppSettings, AppError> {
        self.connection
            .query_row(
                "SELECT locale,theme,timezone,week_starts_on,revision FROM app_settings WHERE workspace_id=?1",
                [DEFAULT_WORKSPACE_ID],
                row_settings,
            )
            .map_err(internal)
    }

    pub fn update_app_settings(
        &mut self,
        next: AppSettings,
        expected_revision: i32,
        operation_id: String,
    ) -> Result<ActionReceipt<AppSettings>, AppError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        let current = tx
            .query_row(
                "SELECT locale,theme,timezone,week_starts_on,revision FROM app_settings WHERE workspace_id=?1",
                [DEFAULT_WORKSPACE_ID],
                row_settings,
            )
            .map_err(internal)?;
        if current.revision != expected_revision {
            return Err(AppError::ConflictRevision {
                entity_id: "app-settings".into(),
                expected: expected_revision,
                actual: current.revision,
            });
        }
        let now = now();
        let changed = tx
            .execute(
                "UPDATE app_settings SET locale=?1,theme=?2,timezone=?3,week_starts_on=?4,revision=?5,updated_at=?6 WHERE workspace_id=?7 AND revision=?8",
                params![
                    next.locale,
                    theme_key(&next.theme),
                    next.timezone,
                    next.week_starts_on,
                    next.revision,
                    now,
                    DEFAULT_WORKSPACE_ID,
                    expected_revision
                ],
            )
            .map_err(internal)?;
        if changed != 1 {
            return Err(AppError::ConflictRevision {
                entity_id: "app-settings".into(),
                expected: expected_revision,
                actual: current.revision,
            });
        }
        let event_id = Uuid::now_v7().to_string();
        tx.execute(
            "INSERT INTO domain_events (id,workspace_id,event_type,aggregate_entity_id,aggregate_revision,payload_json,actor_type,actor_id,operation_id,occurred_at) VALUES (?1,?2,'settings.updated',NULL,?3,?4,'user',?5,?6,?7)",
            params![
                event_id,
                DEFAULT_WORKSPACE_ID,
                next.revision,
                serde_json::json!({"locale": next.locale, "theme": theme_key(&next.theme)}).to_string(),
                DEFAULT_USER_ID,
                operation_id,
                now
            ],
        )
        .map_err(internal)?;
        tx.execute(
            "INSERT INTO audit_events VALUES (?1,?2,'settings.update','user',?3,'[]',?4,NULL,?5)",
            params![
                Uuid::now_v7().to_string(),
                DEFAULT_WORKSPACE_ID,
                DEFAULT_USER_ID,
                operation_id,
                now
            ],
        )
        .map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(ActionReceipt {
            data: next,
            operation_id,
            affected_entity_ids: vec![],
            resulting_revisions: vec![],
            domain_event_ids: vec![event_id],
            undo_batch_id: None,
        })
    }

    pub fn list_areas(&self) -> Result<Vec<Area>, AppError> {
        let mut statement = self.connection.prepare("SELECT id,title,revision,created_at,updated_at,archived_at,deleted_at FROM entities WHERE type_id=?1 AND deleted_at IS NULL AND archived_at IS NULL ORDER BY updated_at DESC").map_err(internal)?;
        statement
            .query_map([AREA_TYPE_ID], row_area)
            .map_err(internal)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(internal)
    }

    pub fn list_trashed_areas(&self) -> Result<Vec<Area>, AppError> {
        let mut statement = self.connection.prepare("SELECT id,title,revision,created_at,updated_at,archived_at,deleted_at FROM entities WHERE type_id=?1 AND deleted_at IS NOT NULL ORDER BY deleted_at DESC").map_err(internal)?;
        statement
            .query_map([AREA_TYPE_ID], row_area)
            .map_err(internal)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(internal)
    }

    pub fn create_area(
        &mut self,
        title: String,
        operation_id: String,
    ) -> Result<ActionReceipt<Area>, AppError> {
        let now = now();
        let area = Area {
            id: Uuid::now_v7().to_string(),
            title,
            revision: 1,
            created_at_ms: now.to_string(),
            updated_at_ms: now.to_string(),
            archived_at_ms: None,
            deleted_at_ms: None,
        };
        let undo_batch_id = Uuid::now_v7().to_string();
        let event_id = Uuid::now_v7().to_string();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        tx.execute("INSERT INTO entities (id,workspace_id,type_id,title,created_at,updated_at,revision,created_by_type,created_by_id,updated_by_type,updated_by_id,origin_device_id) VALUES (?1,?2,?3,?4,?5,?5,1,'user',?6,'user',?6,?7)", params![area.id,DEFAULT_WORKSPACE_ID,AREA_TYPE_ID,area.title,now,DEFAULT_USER_ID,DEFAULT_DEVICE_ID]).map_err(internal)?;
        tx.execute(
            "INSERT INTO areas (entity_id,status) VALUES (?1,'active')",
            [&area.id],
        )
        .map_err(internal)?;
        tx.execute(
            "INSERT INTO entity_search VALUES (?1,?2,'area',?3,?4)",
            params![
                area.id,
                DEFAULT_WORKSPACE_ID,
                area.title,
                normalize_for_search(&area.title)
            ],
        )
        .map_err(internal)?;
        write_history(
            &tx,
            &area,
            "area.created",
            &operation_id,
            &undo_batch_id,
            &event_id,
            now,
        )?;
        tx.execute(
            "INSERT INTO undo_operations VALUES (?1,?2,0,'area.trash',?3)",
            params![
                Uuid::now_v7().to_string(),
                undo_batch_id,
                serde_json::json!({"entityId":area.id,"expectedRevision":1}).to_string()
            ],
        )
        .map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(ActionReceipt {
            data: area.clone(),
            operation_id,
            affected_entity_ids: vec![area.id.clone()],
            resulting_revisions: vec![EntityRevision {
                entity_id: area.id,
                revision: 1,
            }],
            domain_event_ids: vec![event_id],
            undo_batch_id: Some(undo_batch_id),
        })
    }

    pub fn update_area(
        &mut self,
        id: String,
        title: String,
        expected: i32,
        operation_id: String,
    ) -> Result<ActionReceipt<Area>, AppError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        let current = get_area_in_transaction(&tx, &id)?.ok_or(AppError::NotFound {
            entity_id: id.clone(),
        })?;
        if current.revision != expected {
            return Err(AppError::ConflictRevision {
                entity_id: id,
                expected,
                actual: current.revision,
            });
        }
        let now = now();
        let next = Area {
            id: current.id.clone(),
            title,
            revision: current.revision + 1,
            created_at_ms: current.created_at_ms,
            updated_at_ms: now.to_string(),
            archived_at_ms: current.archived_at_ms,
            deleted_at_ms: current.deleted_at_ms,
        };
        let undo_batch_id = Uuid::now_v7().to_string();
        let event_id = Uuid::now_v7().to_string();
        let updated = tx.execute("UPDATE entities SET title=?1,updated_at=?2,revision=?3,updated_by_type='user',updated_by_id=?4 WHERE id=?5 AND revision=?6", params![next.title,now,next.revision,DEFAULT_USER_ID,next.id,current.revision]).map_err(internal)?;
        if updated != 1 {
            return Err(AppError::ConflictRevision {
                entity_id: next.id,
                expected,
                actual: current.revision,
            });
        }
        tx.execute("DELETE FROM entity_search WHERE entity_id=?1", [&next.id])
            .map_err(internal)?;
        tx.execute(
            "INSERT INTO entity_search VALUES (?1,?2,'area',?3,?4)",
            params![
                next.id,
                DEFAULT_WORKSPACE_ID,
                next.title,
                normalize_for_search(&next.title)
            ],
        )
        .map_err(internal)?;
        write_history(
            &tx,
            &next,
            "area.updated",
            &operation_id,
            &undo_batch_id,
            &event_id,
            now,
        )?;
        tx.execute("INSERT INTO undo_operations VALUES (?1,?2,0,'area.restore_title',?3)", params![Uuid::now_v7().to_string(),undo_batch_id,serde_json::json!({"entityId":next.id,"expectedRevision":next.revision,"title":current.title}).to_string()]).map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(ActionReceipt {
            data: next.clone(),
            operation_id,
            affected_entity_ids: vec![next.id.clone()],
            resulting_revisions: vec![EntityRevision {
                entity_id: next.id,
                revision: next.revision,
            }],
            domain_event_ids: vec![event_id],
            undo_batch_id: Some(undo_batch_id),
        })
    }

    pub fn archive_area(
        &mut self,
        id: String,
        expected_revision: i32,
        operation_id: String,
    ) -> Result<ActionReceipt<Area>, AppError> {
        self.change_area_lifecycle(
            id,
            expected_revision,
            operation_id,
            true,
            false,
            "area.archived",
        )
    }

    pub fn trash_area(
        &mut self,
        id: String,
        expected_revision: i32,
        operation_id: String,
    ) -> Result<ActionReceipt<Area>, AppError> {
        self.change_area_lifecycle(
            id,
            expected_revision,
            operation_id,
            false,
            true,
            "area.trashed",
        )
    }

    pub fn restore_area(
        &mut self,
        id: String,
        expected_revision: i32,
        operation_id: String,
    ) -> Result<ActionReceipt<Area>, AppError> {
        self.change_area_lifecycle(
            id,
            expected_revision,
            operation_id,
            false,
            false,
            "area.restored",
        )
    }

    fn change_area_lifecycle(
        &mut self,
        id: String,
        expected_revision: i32,
        operation_id: String,
        archived: bool,
        deleted: bool,
        action: &str,
    ) -> Result<ActionReceipt<Area>, AppError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        let current = get_any_area_in_transaction(&tx, &id)?.ok_or(AppError::NotFound {
            entity_id: id.clone(),
        })?;
        if current.revision != expected_revision {
            return Err(AppError::ConflictRevision {
                entity_id: id,
                expected: expected_revision,
                actual: current.revision,
            });
        }
        if current.archived_at_ms.is_some() == archived
            && current.deleted_at_ms.is_some() == deleted
        {
            return Err(AppError::Validation {
                field: "lifecycle".into(),
                reason: "already in the requested lifecycle state".into(),
            });
        }
        let now = now();
        let next = Area {
            id: current.id.clone(),
            title: current.title.clone(),
            revision: current.revision + 1,
            created_at_ms: current.created_at_ms.clone(),
            updated_at_ms: now.to_string(),
            archived_at_ms: archived.then(|| now.to_string()),
            deleted_at_ms: deleted.then(|| now.to_string()),
        };
        let changed = tx
            .execute(
                "UPDATE entities SET archived_at=?1,deleted_at=?2,updated_at=?3,revision=?4,updated_by_type='user',updated_by_id=?5 WHERE id=?6 AND revision=?7",
                params![
                    next.archived_at_ms.as_deref(),
                    next.deleted_at_ms.as_deref(),
                    now,
                    next.revision,
                    DEFAULT_USER_ID,
                    next.id,
                    current.revision
                ],
            )
            .map_err(internal)?;
        if changed != 1 {
            return Err(AppError::ConflictRevision {
                entity_id: next.id,
                expected: expected_revision,
                actual: current.revision,
            });
        }
        tx.execute("DELETE FROM entity_search WHERE entity_id=?1", [&next.id])
            .map_err(internal)?;
        if !archived && !deleted {
            insert_area_search(&tx, &next)?;
        }
        let undo_batch_id = Uuid::now_v7().to_string();
        let event_id = Uuid::now_v7().to_string();
        write_history(
            &tx,
            &next,
            action,
            &operation_id,
            &undo_batch_id,
            &event_id,
            now,
        )?;
        tx.execute(
            "INSERT INTO undo_operations VALUES (?1,?2,0,'area.restore_lifecycle',?3)",
            params![
                Uuid::now_v7().to_string(),
                undo_batch_id,
                serde_json::json!({
                    "entityId": current.id,
                    "expectedRevision": next.revision,
                    "archived": current.archived_at_ms.is_some(),
                    "deleted": current.deleted_at_ms.is_some()
                })
                .to_string()
            ],
        )
        .map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(ActionReceipt {
            data: next.clone(),
            operation_id,
            affected_entity_ids: vec![next.id.clone()],
            resulting_revisions: vec![EntityRevision {
                entity_id: next.id,
                revision: next.revision,
            }],
            domain_event_ids: vec![event_id],
            undo_batch_id: Some(undo_batch_id),
        })
    }

    pub fn undo(
        &mut self,
        batch: String,
        operation_id: String,
    ) -> Result<ActionReceipt<UndoResult>, AppError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        let (action_key, payload): (String, String) = tx.query_row("SELECT action_key,payload_json FROM undo_operations WHERE undo_batch_id=?1 ORDER BY ordinal LIMIT 1", [&batch], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(internal)?.ok_or(AppError::NotFound { entity_id: batch.clone() })?;
        let available: String = tx
            .query_row(
                "SELECT status FROM undo_batches WHERE id=?1",
                [&batch],
                |r| r.get(0),
            )
            .map_err(internal)?;
        if available != "available" {
            return Err(AppError::Validation {
                field: "undoBatchId".into(),
                reason: "already undone".into(),
            });
        }
        let value: serde_json::Value = serde_json::from_str(&payload).map_err(internal)?;
        let id = value["entityId"]
            .as_str()
            .ok_or_else(|| AppError::IntegrityFailure {
                reason: "invalid undo payload".into(),
            })?
            .to_owned();
        let expected =
            value["expectedRevision"]
                .as_i64()
                .ok_or_else(|| AppError::IntegrityFailure {
                    reason: "invalid undo revision".into(),
                })? as i32;
        let current = get_any_area_in_transaction(&tx, &id)?.ok_or(AppError::NotFound {
            entity_id: id.clone(),
        })?;
        if current.revision != expected {
            return Err(AppError::ConflictRevision {
                entity_id: id,
                expected,
                actual: current.revision,
            });
        }
        let now = now();
        let revision = expected + 1;
        let event_id = Uuid::now_v7().to_string();
        let restored_title = value["title"].as_str();
        let (title, archived, deleted, changed_fields) = match action_key.as_str() {
            "area.restore_title" => (
                restored_title
                    .ok_or_else(|| AppError::IntegrityFailure {
                        reason: "invalid undo title".into(),
                    })?
                    .to_owned(),
                current.archived_at_ms.is_some(),
                current.deleted_at_ms.is_some(),
                "[\"title\"]",
            ),
            "area.trash" => (current.title.clone(), false, true, "[\"deletedAt\"]"),
            "area.restore_lifecycle" => (
                current.title.clone(),
                value["archived"]
                    .as_bool()
                    .ok_or_else(|| AppError::IntegrityFailure {
                        reason: "invalid undo archived state".into(),
                    })?,
                value["deleted"]
                    .as_bool()
                    .ok_or_else(|| AppError::IntegrityFailure {
                        reason: "invalid undo deleted state".into(),
                    })?,
                "[\"archivedAt\",\"deletedAt\"]",
            ),
            _ => {
                return Err(AppError::IntegrityFailure {
                    reason: "unsupported undo action".into(),
                });
            }
        };
        let archived_at_ms = archived.then(|| now.to_string());
        let deleted_at_ms = deleted.then(|| now.to_string());
        let updated = tx
            .execute(
                "UPDATE entities SET title=?1,archived_at=?2,deleted_at=?3,updated_at=?4,revision=?5,updated_by_type='user',updated_by_id=?6 WHERE id=?7 AND revision=?8",
                params![
                    title,
                    archived_at_ms.as_deref(),
                    deleted_at_ms.as_deref(),
                    now,
                    revision,
                    DEFAULT_USER_ID,
                    id,
                    expected
                ],
            )
            .map_err(internal)?;
        tx.execute("DELETE FROM entity_search WHERE entity_id=?1", [&id])
            .map_err(internal)?;
        let resulting_area = Area {
            id: current.id.clone(),
            title,
            revision,
            created_at_ms: current.created_at_ms,
            updated_at_ms: now.to_string(),
            archived_at_ms,
            deleted_at_ms,
        };
        if resulting_area.archived_at_ms.is_none() && resulting_area.deleted_at_ms.is_none() {
            insert_area_search(&tx, &resulting_area)?;
        }
        if updated != 1 {
            return Err(AppError::ConflictRevision {
                entity_id: id,
                expected,
                actual: current.revision,
            });
        }
        let completed_batch = tx.execute(
            "UPDATE undo_batches SET status='undone',undone_at=?1 WHERE id=?2 AND status='available'",
            params![now, batch],
        )
        .map_err(internal)?;
        if completed_batch != 1 {
            return Err(AppError::Validation {
                field: "undoBatchId".into(),
                reason: "already undone".into(),
            });
        }
        write_version(&tx, &resulting_area, changed_fields, &operation_id, now)?;
        tx.execute("INSERT INTO domain_events (id,workspace_id,event_type,aggregate_entity_id,aggregate_revision,payload_json,actor_type,actor_id,operation_id,occurred_at) VALUES (?1,?2,'undo.executed',?3,?4,'{}','user',?5,?6,?7)", params![event_id,DEFAULT_WORKSPACE_ID,id,revision,DEFAULT_USER_ID,operation_id,now]).map_err(internal)?;
        tx.execute(
            "INSERT INTO audit_events VALUES (?1,?2,'undo.execute','user',?3,?4,?5,NULL,?6)",
            params![
                Uuid::now_v7().to_string(),
                DEFAULT_WORKSPACE_ID,
                DEFAULT_USER_ID,
                serde_json::json!([id]).to_string(),
                operation_id,
                now
            ],
        )
        .map_err(internal)?;
        tx.commit().map_err(internal)?;
        Ok(ActionReceipt {
            data: UndoResult {
                undone_undo_batch_id: batch,
                entity_id: id.clone(),
                revision,
            },
            operation_id,
            affected_entity_ids: vec![id.clone()],
            resulting_revisions: vec![EntityRevision {
                entity_id: id,
                revision,
            }],
            domain_event_ids: vec![event_id],
            undo_batch_id: None,
        })
    }

    pub fn search(&self, query: &str, prefix: bool) -> Result<Vec<SearchResult>, AppError> {
        let Some(query) = fts_query(query, prefix) else {
            return Ok(vec![]);
        };
        let mut statement = self.connection.prepare("SELECT entity_id,title,bm25(entity_search) FROM entity_search WHERE entity_search MATCH ?1 ORDER BY bm25(entity_search)").map_err(internal)?;
        statement
            .query_map([query], |r| {
                Ok(SearchResult {
                    entity_id: r.get(0)?,
                    title: r.get(1)?,
                    score: r.get(2)?,
                })
            })
            .map_err(internal)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(internal)
    }

    pub fn area_history(&self, id: &str) -> Result<Vec<AreaVersion>, AppError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT revision,snapshot_json,operation_id,created_at FROM entity_versions WHERE entity_id=?1 ORDER BY revision",
            )
            .map_err(internal)?;
        statement
            .query_map([id], |row| {
                let revision = row.get(0)?;
                let snapshot: String = row.get(1)?;
                let area: Area = serde_json::from_str(&snapshot).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
                Ok(AreaVersion {
                    revision,
                    title: area.title,
                    operation_id: row.get(2)?,
                    created_at_ms: row.get::<_, i64>(3)?.to_string(),
                })
            })
            .map_err(internal)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(internal)
    }

    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        let path = path.as_ref();
        create_new_database_file(path)?;
        let mut destination = Connection::open(path).map_err(internal)?;
        Backup::new(&self.connection, &mut destination)
            .map_err(internal)?
            .run_to_completion(32, Duration::from_millis(1), None)
            .map_err(internal)?;
        destination
            .execute(
                "INSERT INTO backup_metadata VALUES (?1,?2,?3,'ok')",
                params![Uuid::now_v7().to_string(), now(), latest_schema_version()],
            )
            .map_err(internal)?;
        validate_connection(&destination)
    }

    pub fn restore_backup(
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<(), AppError> {
        let source = Connection::open_with_flags(
            source,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(internal)?;
        validate_connection(&source)?;
        create_new_database_file(destination.as_ref())?;
        let mut restored = Connection::open(destination).map_err(internal)?;
        Backup::new(&source, &mut restored)
            .map_err(internal)?
            .run_to_completion(32, Duration::from_millis(1), None)
            .map_err(internal)?;
        validate_connection(&restored)
    }

    pub fn integrity_check(&self) -> Result<bool, AppError> {
        Ok(self
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .map_err(internal)?
            == "ok")
    }
}

fn write_history(
    tx: &rusqlite::Transaction<'_>,
    area: &Area,
    action: &str,
    operation: &str,
    batch: &str,
    event: &str,
    now: i64,
) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO undo_batches VALUES (?1,?2,?3,'available',?4,NULL)",
        params![batch, DEFAULT_WORKSPACE_ID, operation, now],
    )
    .map_err(internal)?;
    write_version(tx, area, "[\"title\"]", operation, now)?;
    tx.execute("INSERT INTO domain_events (id,workspace_id,event_type,aggregate_entity_id,aggregate_revision,payload_json,actor_type,actor_id,operation_id,occurred_at) VALUES (?1,?2,?3,?4,?5,'{}','user',?6,?7,?8)", params![event,DEFAULT_WORKSPACE_ID,action,area.id,area.revision,DEFAULT_USER_ID,operation,now]).map_err(internal)?;
    tx.execute(
        "INSERT INTO audit_events VALUES (?1,?2,?3,'user',?4,?5,?6,?7,?8)",
        params![
            Uuid::now_v7().to_string(),
            DEFAULT_WORKSPACE_ID,
            action,
            DEFAULT_USER_ID,
            serde_json::json!([area.id]).to_string(),
            operation,
            batch,
            now
        ],
    )
    .map_err(internal)?;
    Ok(())
}

fn insert_area_search(tx: &rusqlite::Transaction<'_>, area: &Area) -> Result<(), AppError> {
    tx.execute(
        "INSERT INTO entity_search VALUES (?1,?2,'area',?3,?4)",
        params![
            area.id,
            DEFAULT_WORKSPACE_ID,
            area.title,
            normalize_for_search(&area.title)
        ],
    )
    .map_err(internal)?;
    Ok(())
}

fn row_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<AppSettings> {
    let theme: String = row.get(1)?;
    let theme = match theme.as_str() {
        "light" => ThemePreference::Light,
        "dark" => ThemePreference::Dark,
        "system" => ThemePreference::System,
        _ => {
            return Err(rusqlite::Error::InvalidColumnType(
                1,
                "theme".into(),
                rusqlite::types::Type::Text,
            ));
        }
    };
    Ok(AppSettings {
        locale: row.get(0)?,
        theme,
        timezone: row.get(2)?,
        week_starts_on: row.get(3)?,
        revision: row.get(4)?,
    })
}

fn theme_key(theme: &ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
        ThemePreference::System => "system",
    }
}

fn write_version(
    tx: &rusqlite::Transaction<'_>,
    area: &Area,
    changed_fields: &str,
    operation: &str,
    now: i64,
) -> Result<(), AppError> {
    let snapshot = serde_json::to_string(area).map_err(internal)?;
    tx.execute(
        "INSERT INTO entity_versions VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            Uuid::now_v7().to_string(),
            area.id,
            area.revision,
            snapshot,
            changed_fields,
            operation,
            now
        ],
    )
    .map_err(internal)?;
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), AppError> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL);").map_err(internal)?;
    for (version, sql) in MIGRATIONS {
        let checksum = migration_checksum(sql);
        let stored: Option<String> = connection
            .query_row(
                "SELECT checksum FROM schema_migrations WHERE version=?1",
                [version],
                |r| r.get(0),
            )
            .optional()
            .map_err(internal)?;
        if let Some(stored) = stored {
            if stored != checksum {
                return Err(AppError::IntegrityFailure {
                    reason: "migration checksum mismatch".into(),
                });
            }
            continue;
        }
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Exclusive)
            .map_err(internal)?;
        tx.execute_batch(sql).map_err(internal)?;
        tx.execute(
            "INSERT INTO schema_migrations VALUES (?1,?2,?3)",
            params![version, checksum, now()],
        )
        .map_err(internal)?;
        tx.pragma_update(None, "user_version", version)
            .map_err(internal)?;
        tx.commit().map_err(internal)?;
    }
    Ok(())
}

fn migration_checksum(sql: &str) -> String {
    Sha256::digest(sql.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn latest_schema_version() -> i32 {
    MIGRATIONS.last().map_or(0, |(version, _)| *version)
}

fn validate_connection(connection: &Connection) -> Result<(), AppError> {
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(internal)?;
    if integrity != "ok" {
        return Err(AppError::IntegrityFailure {
            reason: "database integrity check failed".into(),
        });
    }
    let foreign_key_failure: Option<i32> = connection
        .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
        .optional()
        .map_err(internal)?;
    if foreign_key_failure.is_some() {
        return Err(AppError::IntegrityFailure {
            reason: "database foreign key check failed".into(),
        });
    }
    let version: i32 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(internal)?;
    if version != latest_schema_version() {
        return Err(AppError::IntegrityFailure {
            reason: "unsupported database schema version".into(),
        });
    }
    for (migration_version, sql) in MIGRATIONS {
        let stored: Option<String> = connection
            .query_row(
                "SELECT checksum FROM schema_migrations WHERE version=?1",
                [migration_version],
                |row| row.get(0),
            )
            .optional()
            .map_err(internal)?;
        if stored.as_deref() != Some(migration_checksum(sql).as_str()) {
            return Err(AppError::IntegrityFailure {
                reason: "migration checksum mismatch".into(),
            });
        }
    }
    Ok(())
}

fn create_new_database_file(path: &Path) -> Result<(), AppError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map(|_| ())
        .map_err(|_| AppError::Validation {
            field: "destination".into(),
            reason: "must not already exist and must be writable".into(),
        })
}

fn get_area_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
) -> Result<Option<Area>, AppError> {
    tx.query_row("SELECT id,title,revision,created_at,updated_at,archived_at,deleted_at FROM entities WHERE id=?1 AND type_id=?2 AND deleted_at IS NULL AND archived_at IS NULL", params![id,AREA_TYPE_ID], row_area).optional().map_err(internal)
}

fn get_any_area_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
) -> Result<Option<Area>, AppError> {
    tx.query_row("SELECT id,title,revision,created_at,updated_at,archived_at,deleted_at FROM entities WHERE id=?1 AND type_id=?2", params![id,AREA_TYPE_ID], row_area).optional().map_err(internal)
}

fn row_area(row: &rusqlite::Row<'_>) -> rusqlite::Result<Area> {
    Ok(Area {
        id: row.get(0)?,
        title: row.get(1)?,
        revision: row.get(2)?,
        created_at_ms: row.get::<_, i64>(3)?.to_string(),
        updated_at_ms: row.get::<_, i64>(4)?.to_string(),
        archived_at_ms: row.get::<_, Option<i64>>(5)?.map(|value| value.to_string()),
        deleted_at_ms: row.get::<_, Option<i64>>(6)?.map(|value| value.to_string()),
    })
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn internal(_error: impl std::fmt::Display) -> AppError {
    AppError::Internal {
        operation_id: Uuid::now_v7().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn persists_and_undoes_area() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let mut store = EntityStore::open(&path).unwrap();
        let created = store.create_area("Study".into(), "op-1".into()).unwrap();
        assert_eq!(store.list_areas().unwrap().len(), 1);
        assert!(store.integrity_check().unwrap());
        let updated = store
            .update_area(
                created.data.id.clone(),
                "Deep Study".into(),
                1,
                "op-2".into(),
            )
            .unwrap();
        assert!(matches!(
            store.update_area(updated.data.id.clone(), "Stale".into(), 1, "op-3".into()),
            Err(AppError::ConflictRevision { .. })
        ));
        store
            .undo(updated.undo_batch_id.unwrap(), "op-4".into())
            .unwrap();
        assert_eq!(store.list_areas().unwrap()[0].title, "Study");
        let created_second = store.create_area("Health".into(), "op-5".into()).unwrap();
        store
            .undo(created_second.undo_batch_id.unwrap(), "op-6".into())
            .unwrap();
        drop(store);
        assert_eq!(
            EntityStore::open(&path)
                .unwrap()
                .list_areas()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn rolls_back_an_interrupted_initial_migration() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("interrupted.db");
        {
            let mut connection = Connection::open(&path).unwrap();
            connection
                .execute_batch("CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL);")
                .unwrap();
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Exclusive)
                .unwrap();
            transaction.execute_batch(INITIAL_SCHEMA).unwrap();
        }

        let store = EntityStore::open(&path).unwrap();
        assert_eq!(store.health().unwrap().schema_version, 2);
        assert!(store.integrity_check().unwrap());
    }

    #[test]
    fn rejects_a_changed_applied_migration() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("checksum.db");
        drop(EntityStore::open(&path).unwrap());
        Connection::open(&path)
            .unwrap()
            .execute(
                "UPDATE schema_migrations SET checksum='changed' WHERE version=1",
                [],
            )
            .unwrap();

        assert!(matches!(
            EntityStore::open(&path),
            Err(AppError::IntegrityFailure { reason }) if reason == "migration checksum mismatch"
        ));
    }

    #[test]
    fn persists_settings_with_revision_and_audit_history() {
        let dir = tempdir().unwrap();
        let mut store = EntityStore::open(dir.path().join("settings.db")).unwrap();
        let initial = store.app_settings().unwrap();
        assert_eq!(initial.locale, "en");
        let next = AppSettings {
            locale: "ar".into(),
            theme: ThemePreference::Dark,
            timezone: "Asia/Riyadh".into(),
            week_starts_on: 0,
            revision: initial.revision + 1,
        };
        let receipt = store
            .update_app_settings(next.clone(), initial.revision, "settings-1".into())
            .unwrap();
        assert_eq!(receipt.data, next);
        assert!(matches!(
            store.update_app_settings(next, initial.revision, "settings-stale".into()),
            Err(AppError::ConflictRevision { .. })
        ));
        assert_eq!(store.app_settings().unwrap().locale, "ar");
        let audit_count: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE action_key='settings.update'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 1);
    }

    #[test]
    fn archives_trashes_restores_and_undoes_an_area_without_losing_history() {
        let dir = tempdir().unwrap();
        let mut store = EntityStore::open(dir.path().join("lifecycle.db")).unwrap();
        let created = store
            .create_area("Health".into(), "area-create".into())
            .unwrap();
        let archived = store
            .archive_area(
                created.data.id.clone(),
                created.data.revision,
                "area-archive".into(),
            )
            .unwrap();
        assert!(store.list_areas().unwrap().is_empty());
        let restored = store
            .undo(archived.undo_batch_id.unwrap(), "undo-archive".into())
            .unwrap();
        assert_eq!(restored.data.revision, 3);
        let trashed = store
            .trash_area(created.data.id.clone(), 3, "area-trash".into())
            .unwrap();
        assert!(store.list_areas().unwrap().is_empty());
        assert_eq!(store.list_trashed_areas().unwrap().len(), 1);
        let restored = store
            .restore_area(
                created.data.id.clone(),
                trashed.data.revision,
                "area-restore".into(),
            )
            .unwrap();
        assert_eq!(restored.data.revision, 5);
        assert_eq!(store.list_areas().unwrap()[0].title, "Health");
        assert_eq!(store.area_history(&created.data.id).unwrap().len(), 5);
    }
}
