use std::{path::Path, sync::Mutex};

use lifeos_domain::{
    ActionReceipt, AppError, AppSettings, Area, AreaVersion, CONTRACT_VERSION, CreateAreaRequest,
    HealthSnapshot, SearchRequest, SearchResult, UndoRequest, UndoResult, UpdateAppSettingsRequest,
    UpdateAreaRequest,
};
use lifeos_store::EntityStore;

pub struct ApplicationCore {
    store: Mutex<EntityStore>,
}

impl ApplicationCore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        Ok(Self {
            store: Mutex::new(EntityStore::open(path)?),
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
    pub fn create_area(&self, request: CreateAreaRequest) -> Result<ActionReceipt<Area>, AppError> {
        let title = lifeos_domain::validate_title(&request.title)?;
        self.store
            .lock()
            .map_err(|_| internal())?
            .create_area(title, operation_id(request.operation_id))
    }
    pub fn update_area(&self, request: UpdateAreaRequest) -> Result<ActionReceipt<Area>, AppError> {
        let title = lifeos_domain::validate_title(&request.title)?;
        self.store.lock().map_err(|_| internal())?.update_area(
            request.id,
            title,
            request.expected_revision,
            operation_id(request.operation_id),
        )
    }
    pub fn undo(&self, request: UndoRequest) -> Result<ActionReceipt<UndoResult>, AppError> {
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
}
