use std::{path::Path, sync::Mutex};

use lifeos_credentials::{CredentialStore, NativeCredentialStore};
use lifeos_domain::{
    ActionReceipt, ActorKind, AppError, AppSettings, Area, AreaLifecycleRequest, AreaVersion,
    CONTRACT_VERSION, CreateAreaRequest, CredentialReference, HealthSnapshot, PermissionDecision,
    PermissionPolicy, RevokeCredentialRequest, SaveCredentialRequest, SearchRequest, SearchResult,
    UndoRequest, UndoResult, UpdateAppSettingsRequest, UpdateAreaRequest,
    UpsertPermissionPolicyRequest,
};
use lifeos_safety::evaluate;
use lifeos_store::EntityStore;

pub struct ApplicationCore {
    store: Mutex<EntityStore>,
    credentials: Box<dyn CredentialStore>,
}

impl ApplicationCore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        Self::open_with_credentials(path, Box::new(NativeCredentialStore))
    }
    pub fn open_with_credentials(
        path: impl AsRef<Path>,
        credentials: Box<dyn CredentialStore>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            store: Mutex::new(EntityStore::open(path)?),
            credentials,
        })
    }
    pub fn health(&self) -> Result<HealthSnapshot, AppError> {
        self.store.lock().map_err(|_| internal())?.health()
    }
    pub fn app_settings(&self) -> Result<AppSettings, AppError> {
        self.store.lock().map_err(|_| internal())?.app_settings()
    }
    pub fn update_app_settings(
        &self,
        request: UpdateAppSettingsRequest,
    ) -> Result<ActionReceipt<AppSettings>, AppError> {
        self.authorize_local("settings.update")?;
        lifeos_domain::validate_settings(&request)?;
        self.store
            .lock()
            .map_err(|_| internal())?
            .update_app_settings(
                AppSettings {
                    locale: request.locale,
                    theme: request.theme,
                    timezone: request.timezone,
                    week_starts_on: request.week_starts_on,
                    revision: request.expected_revision + 1,
                },
                request.expected_revision,
                operation_id(request.operation_id),
            )
    }
    pub fn list_areas(&self) -> Result<Vec<Area>, AppError> {
        self.store.lock().map_err(|_| internal())?.list_areas()
    }
    pub fn permission_policies(&self) -> Result<Vec<PermissionPolicy>, AppError> {
        self.store
            .lock()
            .map_err(|_| internal())?
            .permission_policies()
    }
    pub fn upsert_permission_policy(
        &self,
        request: UpsertPermissionPolicyRequest,
    ) -> Result<ActionReceipt<PermissionPolicy>, AppError> {
        lifeos_domain::validate_permission_policy(&request)?;
        self.store
            .lock()
            .map_err(|_| internal())?
            .upsert_permission_policy(
                request.subject_kind,
                request.operation.trim().to_owned(),
                request.decision,
                request.enabled,
                request.expected_revision,
                operation_id(request.operation_id),
            )
    }
    pub fn list_trashed_areas(&self) -> Result<Vec<Area>, AppError> {
        self.store
            .lock()
            .map_err(|_| internal())?
            .list_trashed_areas()
    }
    pub fn create_area(&self, request: CreateAreaRequest) -> Result<ActionReceipt<Area>, AppError> {
        self.authorize_local("area.create")?;
        let title = lifeos_domain::validate_title(&request.title)?;
        self.store
            .lock()
            .map_err(|_| internal())?
            .create_area(title, operation_id(request.operation_id))
    }
    pub fn update_area(&self, request: UpdateAreaRequest) -> Result<ActionReceipt<Area>, AppError> {
        self.authorize_local("area.update")?;
        let title = lifeos_domain::validate_title(&request.title)?;
        self.store.lock().map_err(|_| internal())?.update_area(
            request.id,
            title,
            request.expected_revision,
            operation_id(request.operation_id),
        )
    }
    pub fn credential_references(&self) -> Result<Vec<CredentialReference>, AppError> {
        self.store
            .lock()
            .map_err(|_| internal())?
            .credential_references()
    }
    pub fn save_credential(
        &self,
        request: SaveCredentialRequest,
    ) -> Result<ActionReceipt<CredentialReference>, AppError> {
        self.authorize_local("credential.save")?;
        let kind = lifeos_domain::validate_credential_kind(&request.kind)?;
        lifeos_domain::validate_secret(&request.secret)?;
        let reference_id = uuid::Uuid::now_v7().to_string();
        self.credentials.write(&reference_id, &request.secret)?;
        let result = self
            .store
            .lock()
            .map_err(|_| internal())?
            .create_credential_reference(
                reference_id.clone(),
                kind,
                operation_id(request.operation_id),
            );
        if result.is_err() {
            let _ = self.credentials.revoke(&reference_id);
        }
        result
    }
    pub fn revoke_credential(
        &self,
        request: RevokeCredentialRequest,
    ) -> Result<ActionReceipt<CredentialReference>, AppError> {
        self.authorize_local("credential.revoke")?;
        let current = self
            .credential_references()?
            .into_iter()
            .find(|reference| reference.id == request.id)
            .ok_or_else(|| AppError::NotFound {
                entity_id: request.id.clone(),
            })?;
        if current.revision != request.expected_revision {
            return Err(AppError::ConflictRevision {
                entity_id: request.id,
                expected: request.expected_revision,
                actual: current.revision,
            });
        }
        self.credentials.revoke(&current.id)?;
        self.store
            .lock()
            .map_err(|_| internal())?
            .revoke_credential_reference(
                current.id,
                current.revision,
                operation_id(request.operation_id),
            )
    }
    pub fn archive_area(
        &self,
        request: AreaLifecycleRequest,
    ) -> Result<ActionReceipt<Area>, AppError> {
        self.authorize_local("area.archive")?;
        self.store.lock().map_err(|_| internal())?.archive_area(
            request.id,
            request.expected_revision,
            operation_id(request.operation_id),
        )
    }
    pub fn trash_area(
        &self,
        request: AreaLifecycleRequest,
    ) -> Result<ActionReceipt<Area>, AppError> {
        self.authorize_local("area.trash")?;
        self.store.lock().map_err(|_| internal())?.trash_area(
            request.id,
            request.expected_revision,
            operation_id(request.operation_id),
        )
    }
    pub fn restore_area(
        &self,
        request: AreaLifecycleRequest,
    ) -> Result<ActionReceipt<Area>, AppError> {
        self.authorize_local("area.restore")?;
        self.store.lock().map_err(|_| internal())?.restore_area(
            request.id,
            request.expected_revision,
            operation_id(request.operation_id),
        )
    }
    pub fn undo(&self, request: UndoRequest) -> Result<ActionReceipt<UndoResult>, AppError> {
        self.authorize_local("undo.execute")?;
        self.store
            .lock()
            .map_err(|_| internal())?
            .undo(request.undo_batch_id, operation_id(request.operation_id))
    }
    pub fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>, AppError> {
        self.store
            .lock()
            .map_err(|_| internal())?
            .search(&request.query, request.prefix)
    }
    pub fn area_history(&self, id: &str) -> Result<Vec<AreaVersion>, AppError> {
        self.store.lock().map_err(|_| internal())?.area_history(id)
    }
    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        self.store.lock().map_err(|_| internal())?.backup_to(path)
    }
    pub fn restore_backup(
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<(), AppError> {
        EntityStore::restore_backup(source, destination)
    }

    fn authorize_local(&self, operation: &str) -> Result<(), AppError> {
        let policies = self.permission_policies()?;
        match evaluate(ActorKind::User, operation, &policies).decision {
            PermissionDecision::Allow => Ok(()),
            PermissionDecision::Ask => Err(AppError::ConfirmationRequired {
                operation: operation.to_owned(),
            }),
            PermissionDecision::Deny => Err(AppError::PermissionDenied {
                operation: operation.to_owned(),
            }),
        }
    }
}
fn operation_id(value: String) -> String {
    if value.trim().is_empty() {
        uuid::Uuid::now_v7().to_string()
    } else {
        value
    }
}
fn internal() -> AppError {
    AppError::Internal {
        operation_id: format!("core-v{CONTRACT_VERSION}"),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Barrier},
        thread,
    };

    use super::*;
    use tempfile::tempdir;

    fn create(core: &ApplicationCore, title: &str, operation_id: &str) -> ActionReceipt<Area> {
        core.create_area(CreateAreaRequest {
            title: title.into(),
            operation_id: operation_id.into(),
        })
        .unwrap()
    }

    #[test]
    fn lifecycle_persists_every_revision_and_undo_snapshot() {
        let directory = tempdir().unwrap();
        let database = directory.path().join("lifeos.db");
        let core = ApplicationCore::open(&database).unwrap();
        let created = create(&core, "Study", "create-study");
        let updated = core
            .update_area(UpdateAreaRequest {
                id: created.data.id.clone(),
                title: "Deep Study".into(),
                expected_revision: 1,
                operation_id: "update-study".into(),
            })
            .unwrap();
        core.undo(UndoRequest {
            undo_batch_id: updated.undo_batch_id.unwrap(),
            operation_id: "undo-update".into(),
        })
        .unwrap();

        let versions = core.area_history(&created.data.id).unwrap();
        assert_eq!(
            versions
                .iter()
                .map(|version| (version.revision, version.title.as_str()))
                .collect::<Vec<_>>(),
            vec![(1, "Study"), (2, "Deep Study"), (3, "Study")]
        );
        drop(core);

        let reopened = ApplicationCore::open(&database).unwrap();
        assert_eq!(reopened.list_areas().unwrap()[0].title, "Study");
        assert_eq!(reopened.area_history(&created.data.id).unwrap().len(), 3);
    }

    #[test]
    fn only_one_concurrent_revision_checked_update_wins() {
        let directory = tempdir().unwrap();
        let database = directory.path().join("concurrent.db");
        let seed = ApplicationCore::open(&database).unwrap();
        let area = create(&seed, "Original", "seed").data;
        drop(seed);

        let barrier = Arc::new(Barrier::new(3));
        let handles = ["First", "Second"].map(|title| {
            let database = database.clone();
            let area_id = area.id.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let core = ApplicationCore::open(database).unwrap();
                barrier.wait();
                core.update_area(UpdateAreaRequest {
                    id: area_id,
                    title: title.into(),
                    expected_revision: 1,
                    operation_id: format!("update-{title}"),
                })
            })
        });
        barrier.wait();
        let results = handles.map(|handle| handle.join().unwrap());

        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err(AppError::ConflictRevision { .. })))
                .count(),
            1
        );
        let reopened = ApplicationCore::open(&database).unwrap();
        assert_eq!(reopened.area_history(&area.id).unwrap().len(), 2);
    }

    #[test]
    fn backup_restores_to_a_new_valid_database_without_later_changes() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("source.db");
        let backup = directory.path().join("backup.db");
        let restored = directory.path().join("restored.db");
        let core = ApplicationCore::open(&source).unwrap();
        create(&core, "Before backup", "before");
        core.backup_to(&backup).unwrap();
        create(&core, "After backup", "after");
        drop(core);

        ApplicationCore::restore_backup(&backup, &restored).unwrap();
        let restored_core = ApplicationCore::open(&restored).unwrap();
        assert!(restored_core.health().unwrap().foreign_keys_enabled);
        assert_eq!(
            restored_core
                .list_areas()
                .unwrap()
                .iter()
                .map(|area| area.title.as_str())
                .collect::<Vec<_>>(),
            vec!["Before backup"]
        );
        assert!(ApplicationCore::restore_backup(&backup, &restored).is_err());
    }

    #[test]
    fn search_preserves_original_english_arabic_and_mixed_text() {
        let directory = tempdir().unwrap();
        let core = ApplicationCore::open(directory.path().join("search.db")).unwrap();
        for (title, operation) in [
            ("Project Atlas", "english"),
            ("مشروع الحياة", "arabic"),
            ("LifeOS مشروع", "mixed"),
        ] {
            create(&core, title, operation);
        }

        let arabic = core
            .search(SearchRequest {
                query: "الحياة".into(),
                prefix: false,
            })
            .unwrap();
        assert_eq!(arabic[0].title, "مشروع الحياة");

        let english_prefix = core
            .search(SearchRequest {
                query: "Atl".into(),
                prefix: true,
            })
            .unwrap();
        assert_eq!(english_prefix[0].title, "Project Atlas");

        let mixed = core
            .search(SearchRequest {
                query: "LifeOS مشروع".into(),
                prefix: false,
            })
            .unwrap();
        assert_eq!(mixed[0].title, "LifeOS مشروع");
        assert!(mixed.windows(2).all(|pair| pair[0].score <= pair[1].score));
    }

    #[test]
    fn search_orders_more_relevant_fts_matches_first() {
        let directory = tempdir().unwrap();
        let core = ApplicationCore::open(directory.path().join("ranking.db")).unwrap();
        create(&core, "Project planning guide", "long-match");
        create(&core, "Project", "exact-match");

        let results = core
            .search(SearchRequest {
                query: "Project".into(),
                prefix: false,
            })
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Project");
        assert!(results[0].score < results[1].score);
    }

    #[test]
    fn persists_canonical_settings_through_the_core_interface() {
        let directory = tempdir().unwrap();
        let core = ApplicationCore::open(directory.path().join("settings.db")).unwrap();
        let initial = core.app_settings().unwrap();
        let updated = core
            .update_app_settings(UpdateAppSettingsRequest {
                locale: "ar".into(),
                theme: lifeos_domain::ThemePreference::Dark,
                timezone: "Asia/Riyadh".into(),
                week_starts_on: 0,
                expected_revision: initial.revision,
                operation_id: "settings-core".into(),
            })
            .unwrap();
        assert_eq!(updated.data.locale, "ar");
        assert_eq!(updated.data.revision, initial.revision + 1);
    }

    #[test]
    fn permission_policies_block_a_core_mutation_before_the_store_write() {
        let directory = tempdir().unwrap();
        let core = ApplicationCore::open(directory.path().join("policy.db")).unwrap();
        let policy = core
            .upsert_permission_policy(UpsertPermissionPolicyRequest {
                subject_kind: ActorKind::User,
                operation: "area.create".into(),
                decision: PermissionDecision::Deny,
                enabled: true,
                expected_revision: None,
                operation_id: "deny-area-creation".into(),
            })
            .unwrap();

        assert_eq!(policy.data.revision, 1);
        assert!(matches!(
            core.create_area(CreateAreaRequest {
                title: "Blocked".into(),
                operation_id: "blocked-create".into(),
            }),
            Err(AppError::PermissionDenied { operation }) if operation == "area.create"
        ));
        assert!(core.list_areas().unwrap().is_empty());
    }

    #[test]
    fn credential_secret_never_enters_the_canonical_database() {
        let directory = tempdir().unwrap();
        let database = directory.path().join("credentials.db");
        let core = ApplicationCore::open_with_credentials(
            &database,
            Box::new(lifeos_credentials::MemoryCredentialStore::default()),
        )
        .unwrap();
        let reference = core
            .save_credential(SaveCredentialRequest {
                kind: "ai.openai".into(),
                secret: "this-must-never-be-in-sqlite".into(),
                operation_id: "save-test-credential".into(),
            })
            .unwrap();

        assert_eq!(reference.data.kind, "ai.openai");
        assert_eq!(core.credential_references().unwrap(), vec![reference.data]);
        drop(core);
        let database_bytes = std::fs::read(database).unwrap();
        assert!(!String::from_utf8_lossy(&database_bytes).contains("this-must-never-be-in-sqlite"));
    }

    #[test]
    fn credential_revoke_removes_the_secret_and_versions_its_reference() {
        let directory = tempdir().unwrap();
        let store = lifeos_credentials::MemoryCredentialStore::default();
        let core = ApplicationCore::open_with_credentials(
            directory.path().join("revoke.db"),
            Box::new(store.clone()),
        )
        .unwrap();
        let created = core
            .save_credential(SaveCredentialRequest {
                kind: "mcp.token".into(),
                secret: "revoke-me".into(),
                operation_id: "save-for-revoke".into(),
            })
            .unwrap();
        let reference_id = created.data.id.clone();
        let revoked = core
            .revoke_credential(RevokeCredentialRequest {
                id: reference_id.clone(),
                expected_revision: 1,
                operation_id: "revoke-test-credential".into(),
            })
            .unwrap();

        assert_eq!(revoked.data.revision, 2);
        assert!(revoked.data.revoked_at_ms.is_some());
        assert_eq!(core.credential_references().unwrap(), vec![revoked.data]);
        assert!(matches!(
            store.read(&reference_id),
            Err(AppError::Unavailable { .. })
        ));
    }
}
