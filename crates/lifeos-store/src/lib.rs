use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use lifeos_domain::{
    AREA_TYPE_ID, ActionReceipt, AppError, Area, DEFAULT_DEVICE_ID, DEFAULT_USER_ID,
    DEFAULT_WORKSPACE_ID, EntityRevision, HealthSnapshot, SearchResult, UndoResult,
};
use lifeos_search::{fts_query, normalize_for_search};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, backup::Backup, params};
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

    pub fn list_areas(&self) -> Result<Vec<Area>, AppError> {
        let mut statement = self.connection.prepare("SELECT id,title,revision,created_at,updated_at FROM entities WHERE type_id=?1 AND deleted_at IS NULL ORDER BY updated_at DESC").map_err(internal)?;
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
        let current = self.get_area(&id)?.ok_or(AppError::NotFound {
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
        };
        let undo_batch_id = Uuid::now_v7().to_string();
        let event_id = Uuid::now_v7().to_string();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        tx.execute("UPDATE entities SET title=?1,updated_at=?2,revision=?3,updated_by_type='user',updated_by_id=?4 WHERE id=?5 AND revision=?6", params![next.title,now,next.revision,DEFAULT_USER_ID,next.id,current.revision]).map_err(internal)?;
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

    pub fn undo(
        &mut self,
        batch: String,
        operation_id: String,
    ) -> Result<ActionReceipt<UndoResult>, AppError> {
        let payload: String = self.connection.query_row("SELECT payload_json FROM undo_operations WHERE undo_batch_id=?1 ORDER BY ordinal LIMIT 1", [&batch], |r| r.get(0)).optional().map_err(internal)?.ok_or(AppError::NotFound { entity_id: batch.clone() })?;
        let available: String = self
            .connection
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
        let current = self.get_area(&id)?.ok_or(AppError::NotFound {
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
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(internal)?;
        if let Some(title) = value["title"].as_str() {
            tx.execute(
                "UPDATE entities SET title=?1,updated_at=?2,revision=?3 WHERE id=?4",
                params![title, now, revision, id],
            )
            .map_err(internal)?;
            tx.execute("DELETE FROM entity_search WHERE entity_id=?1", [&id])
                .map_err(internal)?;
            tx.execute(
                "INSERT INTO entity_search VALUES (?1,?2,'area',?3,?4)",
                params![id, DEFAULT_WORKSPACE_ID, title, normalize_for_search(title)],
            )
            .map_err(internal)?;
        } else {
            tx.execute(
                "UPDATE entities SET deleted_at=?1,updated_at=?1,revision=?2 WHERE id=?3",
                params![now, revision, id],
            )
            .map_err(internal)?;
            tx.execute("DELETE FROM entity_search WHERE entity_id=?1", [&id])
                .map_err(internal)?;
        }
        tx.execute(
            "UPDATE undo_batches SET status='undone',undone_at=?1 WHERE id=?2",
            params![now, batch],
        )
        .map_err(internal)?;
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

    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        let mut destination = Connection::open(path).map_err(internal)?;
        Backup::new(&self.connection, &mut destination)
            .map_err(internal)?
            .run_to_completion(32, Duration::from_millis(1), None)
            .map_err(internal)
    }

    pub fn integrity_check(&self) -> Result<bool, AppError> {
        Ok(self
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .map_err(internal)?
            == "ok")
    }

    fn get_area(&self, id: &str) -> Result<Option<Area>, AppError> {
        self.connection.query_row("SELECT id,title,revision,created_at,updated_at FROM entities WHERE id=?1 AND type_id=?2 AND deleted_at IS NULL", params![id,AREA_TYPE_ID], row_area).optional().map_err(internal)
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
    let snapshot = serde_json::to_string(area).map_err(internal)?;
    tx.execute(
        "INSERT INTO undo_batches VALUES (?1,?2,?3,'available',?4,NULL)",
        params![batch, DEFAULT_WORKSPACE_ID, operation, now],
    )
    .map_err(internal)?;
    tx.execute(
        "INSERT INTO entity_versions VALUES (?1,?2,?3,?4,'[\"title\"]',?5,?6)",
        params![
            Uuid::now_v7().to_string(),
            area.id,
            area.revision,
            snapshot,
            operation,
            now
        ],
    )
    .map_err(internal)?;
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
fn migrate(connection: &mut Connection) -> Result<(), AppError> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL);").map_err(internal)?;
    let checksum = Sha256::digest(INITIAL_SCHEMA.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let stored: Option<String> = connection
        .query_row(
            "SELECT checksum FROM schema_migrations WHERE version=1",
            [],
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
        return Ok(());
    }
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Exclusive)
        .map_err(internal)?;
    tx.execute_batch(INITIAL_SCHEMA).map_err(internal)?;
    tx.execute(
        "INSERT INTO schema_migrations VALUES (1,?1,?2)",
        params![checksum, now()],
    )
    .map_err(internal)?;
    tx.pragma_update(None, "user_version", 1)
        .map_err(internal)?;
    tx.commit().map_err(internal)
}
fn row_area(row: &rusqlite::Row<'_>) -> rusqlite::Result<Area> {
    Ok(Area {
        id: row.get(0)?,
        title: row.get(1)?,
        revision: row.get(2)?,
        created_at_ms: row.get::<_, i64>(3)?.to_string(),
        updated_at_ms: row.get::<_, i64>(4)?.to_string(),
    })
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn internal(error: impl std::fmt::Display) -> AppError {
    AppError::IntegrityFailure {
        reason: error.to_string(),
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
}
