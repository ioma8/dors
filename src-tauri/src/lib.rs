use tauri::State;

pub mod adapters;
pub mod app_state;
pub mod config;
pub mod domain;
pub mod services;

use app_state::AppState;

#[tauri::command]
fn get_dock_state(state: State<'_, AppState>) -> Result<Vec<domain::DockItemView>, String> {
    state.dock_items()
}

#[tauri::command]
fn trigger_launch(request: services::launcher::LaunchRequest) -> services::launcher::LaunchAction {
    services::launcher::launch_or_activate(
        &request,
        |_request| services::launcher::LaunchResult::ActivationFailed,
        |_request| services::launcher::LaunchResult::Launched,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(Vec::new()))
        .invoke_handler(tauri::generate_handler![get_dock_state, trigger_launch])
        .run(tauri::generate_context!())
}
