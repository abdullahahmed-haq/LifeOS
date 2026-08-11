use serde::{Deserialize, Serialize};
use specta::Type;

pub const CONTRACT_VERSION: u16 = 1;
pub const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-7000-8000-000000000001";
pub const DEFAULT_USER_ID: &str = "00000000-0000-7000-8000-000000000002";
pub const DEFAULT_DEVICE_ID: &str = "00000000-0000-7000-8000-000000000003";
pub const AREA_TYPE_ID: &str = "00000000-0000-7000-8000-000000000004";

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    pub id: String,
    pub title: String,
    pub revision: i32,
    pub created_at_ms: String,
    pub updated_at_ms: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateAreaRequest {
    pub title: String,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAreaRequest {
    pub id: String,
    pub title: String,
    pub expected_revision: i32,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UndoRequest {
    pub undo_batch_id: String,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ActionReceipt<T> {
    pub data: T,
    pub operation_id: String,
    pub affected_entity_ids: Vec<String>,
    pub resulting_revisions: Vec<EntityRevision>,
    pub domain_event_ids: Vec<String>,
    pub undo_batch_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EntityRevision {
    pub entity_id: String,
    pub revision: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub undone_undo_batch_id: String,
    pub entity_id: String,
    pub revision: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshot {
    pub schema_version: i32,
    pub sqlite_version: String,
    pub journal_mode: String,
    pub foreign_keys_enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub prefix: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entity_id: String,
    pub title: String,
    pub score: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(tag = "code", content = "details", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppError {
    Validation {
        field: String,
        reason: String,
    },
    NotFound {
        entity_id: String,
    },
    ConflictRevision {
        entity_id: String,
        expected: i32,
        actual: i32,
    },
    IntegrityFailure {
        reason: String,
    },
    Internal {
        operation_id: String,
    },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AppError {}

pub fn validate_title(value: &str) -> Result<String, AppError> {
    let title = value.trim();
    if title.is_empty() || title.chars().count() > 200 || title.chars().any(char::is_control) {
        return Err(AppError::Validation {
            field: "title".into(),
            reason: "must be 1-200 visible characters".into(),
        });
    }
    Ok(title.to_owned())
}
