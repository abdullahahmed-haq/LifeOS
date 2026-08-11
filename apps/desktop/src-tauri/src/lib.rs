use std::{fs, path::PathBuf};

use lifeos_core::ApplicationCore;
use lifeos_domain::{
    ActionReceipt, AppError, Area, CreateAreaRequest, HealthSnapshot, SearchRequest, SearchResult,
    UndoRequest, UndoResult, UpdateAreaRequest,
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
fn list_areas(state: State<'_, AppState>) -> Result<Vec<Area>, AppError> {
    state.core.list_areas()
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
            list_areas,
            create_area,
            update_area,
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

pub fn run() {
    let ipc = ipc_builder();
    #[cfg(debug_assertions)]
    export_bindings(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../packages/contracts/src/bindings.ts"),
    )
    .expect("generate contracts");
    let event_ipc = ipc.clone();
    tauri::Builder::default()
        .setup(move |app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("application data directory");
            fs::create_dir_all(&data_dir).expect("application data directory exists");
            let database = std::env::var_os("LIFEOS_DEV_DATABASE")
                .map(PathBuf::from)
                .unwrap_or_else(|| data_dir.join("lifeos.db"));
            app.manage(AppState {
                core: ApplicationCore::open(database).expect("open LifeOS database"),
            });
            event_ipc.mount_events(app);
            Ok(())
        })
        .invoke_handler(ipc.invoke_handler())
        .run(tauri::generate_context!())
        .expect("run LifeOS");
}
