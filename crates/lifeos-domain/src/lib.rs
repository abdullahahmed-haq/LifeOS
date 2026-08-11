use serde::{Deserialize, Serialize};
use specta::Type;

pub const CONTRACT_VERSION: u16 = 1;
pub const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-7000-8000-000000000001";
pub const DEFAULT_USER_ID: &str = "00000000-0000-7000-8000-000000000002";
pub const DEFAULT_DEVICE_ID: &str = "00000000-0000-7000-8000-000000000003";
pub const AREA_TYPE_ID: &str = "00000000-0000-7000-8000-000000000004";

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub locale: String,
    pub theme: ThemePreference,
    pub timezone: String,
    pub week_starts_on: u8,
    pub revision: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppSettingsRequest {
    pub locale: String,
    pub theme: ThemePreference,
    pub timezone: String,
    pub week_starts_on: u8,
    pub expected_revision: i32,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    pub id: String,
    pub title: String,
    pub revision: i32,
    pub created_at_ms: String,
    pub updated_at_ms: String,
    pub archived_at_ms: Option<String>,
    pub deleted_at_ms: Option<String>,
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
pub struct AreaLifecycleRequest {
    pub id: String,
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
#[serde(rename_all = "camelCase")]
pub struct AreaVersion {
    pub revision: i32,
    pub title: String,
    pub operation_id: String,
    pub created_at_ms: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(
    tag = "code",
    content = "details",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
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
    PermissionDenied {
        operation: String,
    },
    Unavailable {
        service: String,
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

pub fn validate_settings(request: &UpdateAppSettingsRequest) -> Result<(), AppError> {
    if !matches!(request.locale.as_str(), "en" | "ar") {
        return Err(AppError::Validation {
            field: "locale".into(),
            reason: "must be a supported locale".into(),
        });
    }
    if request.timezone.trim().is_empty() || request.timezone.len() > 100 {
        return Err(AppError::Validation {
            field: "timezone".into(),
            reason: "must be a valid configured timezone identifier".into(),
        });
    }
    if request.week_starts_on > 6 {
        return Err(AppError::Validation {
            field: "weekStartsOn".into(),
            reason: "must be between 0 and 6".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_errors_serialize_with_camel_case_details() {
        let error = AppError::ConflictRevision {
            entity_id: "area-1".into(),
            expected: 2,
            actual: 3,
        };

        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::json!({
                "code": "CONFLICT_REVISION",
                "details": { "entityId": "area-1", "expected": 2, "actual": 3 }
            })
        );
    }

    #[test]
    fn settings_reject_unsupported_locale_and_week_start() {
        let mut request = UpdateAppSettingsRequest {
            locale: "fr".into(),
            theme: ThemePreference::System,
            timezone: "Africa/Cairo".into(),
            week_starts_on: 1,
            expected_revision: 1,
            operation_id: "settings-test".into(),
        };
        assert!(matches!(
            validate_settings(&request),
            Err(AppError::Validation { .. })
        ));
        request.locale = "en".into();
        request.week_starts_on = 7;
        assert!(matches!(
            validate_settings(&request),
            Err(AppError::Validation { .. })
        ));
    }
}
