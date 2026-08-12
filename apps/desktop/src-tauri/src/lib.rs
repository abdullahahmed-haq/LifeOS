use std::{fs, path::PathBuf};

use lifeos_core::ApplicationCore;
use lifeos_domain::{
    ActionReceipt, AppError, AppSettings, Area, AreaLifecycleRequest, CreateAreaRequest,
    HealthSnapshot, PermissionPolicy, SearchRequest, SearchResult, UndoRequest, UndoResult,
    UpdateAppSettingsRequest, UpdateAreaRequest, UpsertPermissionPolicyRequest,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use tauri::{Manager, State};
use tauri_specta::{Builder, Event, collect_commands, collect_events};

pub struct AppState {
    core: ApplicationCore,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type, Event)]
pub struct FoundationProgress {
    pub phase: String,
    pub percent: u8,
}

#[tauri::command]
#[specta::specta]
fn core_health(state: State<'_, AppState>) -> Result<HealthSnapshot, AppError> {
    state.core.health()
}
#[tauri::command]
#[specta::specta]
fn app_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    state.core.app_settings()
}
#[tauri::command]
#[specta::specta]
fn update_app_settings(
    state: State<'_, AppState>,
    request: UpdateAppSettingsRequest,
) -> Result<ActionReceipt<AppSettings>, AppError> {
    state.core.update_app_settings(request)
}
#[tauri::command]
#[specta::specta]
fn list_permission_policies(state: State<'_, AppState>) -> Result<Vec<PermissionPolicy>, AppError> {
    state.core.permission_policies()
}
#[tauri::command]
#[specta::specta]
fn upsert_permission_policy(
    state: State<'_, AppState>,
    request: UpsertPermissionPolicyRequest,
) -> Result<ActionReceipt<PermissionPolicy>, AppError> {
    state.core.upsert_permission_policy(request)
}
#[tauri::command]
#[specta::specta]
fn list_areas(state: State<'_, AppState>) -> Result<Vec<Area>, AppError> {
    state.core.list_areas()
}
#[tauri::command]
#[specta::specta]
fn list_trashed_areas(state: State<'_, AppState>) -> Result<Vec<Area>, AppError> {
    state.core.list_trashed_areas()
}
#[tauri::command]
#[specta::specta]
fn create_area(
    state: State<'_, AppState>,
    request: CreateAreaRequest,
) -> Result<ActionReceipt<Area>, AppError> {
    state.core.create_area(request)
}
#[tauri::command]
#[specta::specta]
fn update_area(
    state: State<'_, AppState>,
    request: UpdateAreaRequest,
) -> Result<ActionReceipt<Area>, AppError> {
    state.core.update_area(request)
}
#[tauri::command]
#[specta::specta]
fn archive_area(
    state: State<'_, AppState>,
    request: AreaLifecycleRequest,
) -> Result<ActionReceipt<Area>, AppError> {
    state.core.archive_area(request)
}
#[tauri::command]
#[specta::specta]
fn trash_area(
    state: State<'_, AppState>,
    request: AreaLifecycleRequest,
) -> Result<ActionReceipt<Area>, AppError> {
    state.core.trash_area(request)
}
#[tauri::command]
#[specta::specta]
fn restore_area(
    state: State<'_, AppState>,
    request: AreaLifecycleRequest,
) -> Result<ActionReceipt<Area>, AppError> {
    state.core.restore_area(request)
}
#[tauri::command]
#[specta::specta]
fn undo_action(
    state: State<'_, AppState>,
    request: UndoRequest,
) -> Result<ActionReceipt<UndoResult>, AppError> {
    state.core.undo(request)
}
#[tauri::command]
#[specta::specta]
fn search_entities(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<Vec<SearchResult>, AppError> {
    state.core.search(request)
}
#[tauri::command]
#[specta::specta]
fn foundation_check(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<HealthSnapshot, AppError> {
    FoundationProgress {
        phase: "checking".into(),
        percent: 50,
    }
    .emit(&app)
    .map_err(|_| AppError::Internal {
        operation_id: "foundation-event".into(),
    })?;
    let health = state.core.health()?;
    FoundationProgress {
        phase: "ready".into(),
        percent: 100,
    }
    .emit(&app)
    .map_err(|_| AppError::Internal {
        operation_id: "foundation-event".into(),
    })?;
    Ok(health)
}

fn ipc_builder() -> Builder<tauri::Wry> {
    Builder::new()
        .commands(collect_commands![
            core_health,
            app_settings,
            update_app_settings,
            list_permission_policies,
            upsert_permission_policy,
            list_areas,
            list_trashed_areas,
            create_area,
            update_area,
            archive_area,
            trash_area,
            restore_area,
            undo_action,
            search_entities,
            foundation_check
        ])
        .events(collect_events![FoundationProgress])
}

pub fn export_bindings(
    path: impl AsRef<std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    ipc_builder().export(Typescript::default(), path)?;
    Ok(())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ipc = ipc_builder();
    #[cfg(debug_assertions)]
    export_bindings(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../packages/contracts/src/bindings.ts"),
    )?;
    let event_ipc = ipc.clone();
    tauri::Builder::default()
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&data_dir)?;
            let database = database_path(data_dir);
            app.manage(AppState {
                core: ApplicationCore::open(database)?,
            });
            event_ipc.mount_events(app);
            Ok(())
        })
        .invoke_handler(ipc.invoke_handler())
        .run(tauri::generate_context!())?;
    Ok(())
}

fn database_path(data_dir: PathBuf) -> PathBuf {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os("LIFEOS_DEV_DATABASE") {
        return PathBuf::from(path);
    }
    data_dir.join("lifeos.db")
}
