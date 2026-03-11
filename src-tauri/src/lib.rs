use tauri::{Manager, PhysicalPosition, Position, State};

pub mod adapters;
pub mod app_state;
pub mod config;
pub mod domain;
pub mod native_app;
pub mod services;
pub mod window_level;
pub mod window_position;

use app_state::AppState;
use config::{ConfigLoad, ConfigStore, DockConfig};

#[tauri::command]
fn get_dock_state(state: State<'_, AppState>) -> Result<Vec<domain::DockItemView>, String> {
    let running_apps = adapters::running_apps::read_running_apps()?;
    let dock_items = state.refresh_with_running_apps(running_apps)?;
    Ok(dock_items
        .into_iter()
        .map(|mut item| {
            item.icon_src =
                adapters::icon_loader::load_icon_data_url(&item.identity.path).unwrap_or_default();
            item
        })
        .collect())
}

#[tauri::command]
fn trigger_launch(request: services::launcher::LaunchRequest) -> services::launcher::LaunchAction {
    services::launcher::launch_or_activate(
        &request,
        services::launcher::activate_app,
        services::launcher::launch_app,
    )
}

#[tauri::command]
fn debug_log(message: String) {
    eprintln!("[dors-debug] {message}");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            configure_overlay_app()?;
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            let config_store = ConfigStore::new(app_data_dir);
            let load = config_store
                .load()
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            let config = bootstrap_config(load, &config_store);
            let state = AppState::new(config);
            let running_apps = adapters::running_apps::read_running_apps().unwrap_or_default();
            let _ = state.refresh_with_running_apps(running_apps);
            app.manage(state);
            position_main_window(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_log,
            get_dock_state,
            trigger_launch
        ])
        .run(tauri::generate_context!())
}

fn bootstrap_config(load: ConfigLoad, config_store: &ConfigStore) -> DockConfig {
    match app_state::bootstrap_pinned_apps(
        load,
        || adapters::dock_import::DockImportReader.read_current_user(),
        |config| config_store.save(config),
    ) {
        Ok(result) => result.config,
        Err(_) => DockConfig::default(),
    }
}

fn position_main_window<R: tauri::Runtime>(
    app: &tauri::App<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    let monitor = window
        .current_monitor()?
        .or(window.primary_monitor()?)
        .ok_or("no monitor available")?;
    let window_size = window.outer_size()?;
    let placement = window_position::bottom_center_placement(
        monitor.position().x,
        monitor.position().y,
        monitor.size().width,
        monitor.size().height,
        window_size.width,
        window_size.height,
        0,
    );

    window.set_position(Position::Physical(PhysicalPosition::new(
        placement.x,
        placement.y,
    )))?;
    configure_overlay_window(&window)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_overlay_app() -> Result<(), Box<dyn std::error::Error>> {
    let marker = objc2_foundation::MainThreadMarker::new()
        .ok_or("overlay app configuration must run on the main thread")?;
    let ns_app = objc2_app_kit::NSApplication::sharedApplication(marker);
    let _ = ns_app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
    ns_app.deactivate();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn configure_overlay_app() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_overlay_window<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    window.with_webview(|webview| unsafe {
        let ns_window: &objc2_app_kit::NSWindow = &*webview.ns_window().cast();
        let style_mask = ns_window.styleMask();
        ns_window.setStyleMask(objc2_app_kit::NSWindowStyleMask(
            window_level::overlay_style_mask(style_mask.0),
        ));
        ns_window.setLevel(window_level::OVERLAY_WINDOW_LEVEL);
        ns_window.setHidesOnDeactivate(false);
    })?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn configure_overlay_window<R: tauri::Runtime>(
    _window: &tauri::WebviewWindow<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
