use serde::{Deserialize, Serialize};
use specta::Type;

pub const CONTRACT_VERSION: u16 = 1;
pub const DEFAULT_WORKSPACE_ID: &str = "00000000-0000-7000-8000-000000000001";
pub const DEFAULT_USER_ID: &str = "00000000-0000-7000-8000-000000000002";
pub const DEFAULT_DEVICE_ID: &str = "00000000-0000-7000-8000-000000000003";
pub const AREA_TYPE_ID: &str = "00000000-0000-7000-8000-000000000004";
pub const GOAL_TYPE_ID: &str = "00000000-0000-7000-8000-000000000005";
pub const PROJECT_TYPE_ID: &str = "00000000-0000-7000-8000-000000000006";

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Ai,
    Mcp,
    Automation,
    Obsidian,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    Allow,
    Ask,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPolicy {
    pub id: String,
    pub subject_kind: ActorKind,
    pub operation: String,
    pub decision: PermissionDecision,
    pub enabled: bool,
    pub revision: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionEvaluation {
    pub actor: ActorKind,
    pub operation: String,
    pub decision: PermissionDecision,
    pub matched_policy_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CredentialReference {
    pub id: String,
    pub kind: String,
    pub revision: i32,
    pub revoked_at_ms: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SaveCredentialRequest {
    pub kind: String,
    pub secret: String,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RevokeCredentialRequest {
    pub id: String,
    pub expected_revision: i32,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPermissionPolicyRequest {
    pub subject_kind: ActorKind,
    pub operation: String,
    pub decision: PermissionDecision,
    pub enabled: bool,
    pub expected_revision: Option<i32>,
    pub operation_id: String,
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

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalHorizon {
    Short,
    Medium,
    Long,
    Lifetime,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub horizon: GoalHorizon,
    pub status: String,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub revision: i32,
    pub created_at_ms: String,
    pub updated_at_ms: String,
    pub archived_at_ms: Option<String>,
    pub deleted_at_ms: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalRequest {
    pub title: String,
    pub horizon: GoalHorizon,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalRequest {
    pub id: String,
    pub title: String,
    pub horizon: GoalHorizon,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub expected_revision: i32,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    pub parent_project_id: Option<String>,
    pub status: String,
    pub priority: Option<u8>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub revision: i32,
    pub created_at_ms: String,
    pub updated_at_ms: String,
    pub archived_at_ms: Option<String>,
    pub deleted_at_ms: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub title: String,
    pub parent_project_id: Option<String>,
    pub priority: Option<u8>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub operation_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub id: String,
    pub title: String,
    pub priority: Option<u8>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
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
#[serde(rename_all = "camelCase")]
pub struct AreaHistoryRequest {
    pub id: String,
    pub limit: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AuditListRequest {
    pub limit: u8,
}

/// A deliberately narrow audit projection for the user-facing timeline.
/// Sensitive audit payloads, entity lists, and internal operation identifiers
/// stay inside Rust Core; callers receive only safe, display-oriented metadata.
#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: String,
    pub action: String,
    pub actor_kind: ActorKind,
    pub occurred_at_ms: String,
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
    ConflictExternal {
        reason: String,
    },
    IntegrityFailure {
        reason: String,
    },
    PermissionDenied {
        operation: String,
    },
    ConfirmationRequired {
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

pub fn validate_goal_dates(
    start_date: &Option<String>,
    target_date: &Option<String>,
) -> Result<(Option<String>, Option<String>), AppError> {
    let start_date = validate_optional_local_date(start_date, "startDate")?;
    let target_date = validate_optional_local_date(target_date, "targetDate")?;
    if matches!((&start_date, &target_date), (Some(start), Some(target)) if target < start) {
        return Err(AppError::Validation {
            field: "targetDate".into(),
            reason: "must not be before startDate".into(),
        });
    }
    Ok((start_date, target_date))
}

pub fn validate_priority(priority: Option<u8>) -> Result<Option<u8>, AppError> {
    if matches!(priority, Some(value) if value > 100) {
        return Err(AppError::Validation {
            field: "priority".into(),
            reason: "must be between 0 and 100".into(),
        });
    }
    Ok(priority)
}

fn validate_optional_local_date(
    value: &Option<String>,
    field: &str,
) -> Result<Option<String>, AppError> {
    let Some(value) = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let shape_is_valid = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit());
    if !shape_is_valid {
        return Err(AppError::Validation {
            field: field.into(),
            reason: "must use ISO local-date format YYYY-MM-DD".into(),
        });
    }
    let year = value[0..4].parse::<u16>().unwrap_or_default();
    let month = value[5..7].parse::<u8>().unwrap_or_default();
    let day = value[8..10].parse::<u8>().unwrap_or_default();
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > max_day {
        return Err(AppError::Validation {
            field: field.into(),
            reason: "must be a valid calendar date".into(),
        });
    }
    Ok(Some(value.to_owned()))
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

pub fn validate_permission_policy(request: &UpsertPermissionPolicyRequest) -> Result<(), AppError> {
    let operation = request.operation.trim();
    if operation.is_empty()
        || operation.len() > 100
        || !operation
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '.')
    {
        return Err(AppError::Validation {
            field: "operation".into(),
            reason: "must be a lowercase dotted operation key".into(),
        });
    }
    if request
        .expected_revision
        .is_some_and(|revision| revision < 1)
    {
        return Err(AppError::Validation {
            field: "expectedRevision".into(),
            reason: "must be at least 1 when provided".into(),
        });
    }
    Ok(())
}

pub fn validate_credential_kind(value: &str) -> Result<String, AppError> {
    let kind = value.trim();
    if kind.is_empty()
        || kind.len() > 100
        || !kind
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '_' || character == '.')
    {
        return Err(AppError::Validation {
            field: "credentialKind".into(),
            reason: "must be a lowercase dotted or underscored key".into(),
        });
    }
    Ok(kind.to_owned())
}

pub fn validate_secret(value: &str) -> Result<(), AppError> {
    if value.is_empty() || value.len() > 16_384 {
        return Err(AppError::Validation {
            field: "secret".into(),
            reason: "must be between 1 and 16384 bytes".into(),
        });
    }
    Ok(())
}

pub fn validate_page_limit(limit: u8) -> Result<usize, AppError> {
    if limit == 0 || limit > 100 {
        return Err(AppError::Validation {
            field: "limit".into(),
            reason: "must be between 1 and 100".into(),
        });
    }
    Ok(usize::from(limit))
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

    #[test]
    fn page_limits_are_bounded_for_public_history_queries() {
        assert_eq!(validate_page_limit(1).unwrap(), 1);
        assert_eq!(validate_page_limit(100).unwrap(), 100);
        assert!(matches!(
            validate_page_limit(0),
            Err(AppError::Validation { field, .. }) if field == "limit"
        ));
    }

    #[test]
    fn goal_dates_require_real_ordered_local_calendar_dates() {
        assert_eq!(
            validate_goal_dates(&Some("2024-02-29".into()), &Some("2024-03-01".into())).unwrap(),
            (Some("2024-02-29".into()), Some("2024-03-01".into()))
        );
        assert!(matches!(
            validate_goal_dates(&Some("2025-02-29".into()), &None),
            Err(AppError::Validation { field, .. }) if field == "startDate"
        ));
        assert!(matches!(
            validate_goal_dates(&Some("2026-08-13".into()), &Some("2026-08-12".into())),
            Err(AppError::Validation { field, .. }) if field == "targetDate"
        ));
    }
}
