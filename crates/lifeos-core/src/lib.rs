use std::{path::Path, sync::Mutex};

use lifeos_domain::{
    ActionReceipt, AppError, Area, CONTRACT_VERSION, CreateAreaRequest, HealthSnapshot,
    SearchRequest, SearchResult, UndoRequest, UndoResult, UpdateAreaRequest,
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
    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        self.store.lock().map_err(|_| internal())?.backup_to(path)
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
