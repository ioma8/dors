use std::path::{Path, PathBuf};

use crate::adapters::{dock_import::DockImportReader, icon_loader, running_apps};
use crate::app_state::{self, AppState};
use crate::config::{ConfigLoad, ConfigStore, DockConfig};
use crate::native_app::view_model::{NativeDockItemModel, build_models};

pub fn refresh_models_and_clamp<LoadModels, ClampWindows>(
    load_models: LoadModels,
    clamp_windows: ClampWindows,
) -> Result<Vec<NativeDockItemModel>, String>
where
    LoadModels: FnOnce() -> Result<Vec<NativeDockItemModel>, String>,
    ClampWindows: FnOnce() -> Result<(), String>,
{
    let models = load_models()?;
    let _ = clamp_windows();
    Ok(models)
}

pub fn build_refresh_models<LoadIcon>(
    state: &AppState,
    running_apps: Vec<crate::domain::RunningApp>,
    load_icon: LoadIcon,
) -> Result<Vec<NativeDockItemModel>, String>
where
    LoadIcon: Fn(&Path) -> Option<String>,
{
    let mut items = state.refresh_snapshot(running_apps)?;
    for item in &mut items {
        item.icon_src = load_icon(&item.identity.path).unwrap_or_default();
    }
    Ok(build_models(&items))
}

pub fn load_startup_models() -> Result<Vec<NativeDockItemModel>, String> {
    let app_support_dir = native_app_support_dir()?;
    let config_store = ConfigStore::new(app_support_dir);
    let load = config_store.load().map_err(|error| error.to_string())?;
    let config = bootstrap_config(load, &config_store)?;
    let state = AppState::new(config);
    let running = running_apps::read_running_apps()?;

    build_refresh_models(&state, running, icon_loader::load_icon_data_url)
}

fn bootstrap_config(load: ConfigLoad, config_store: &ConfigStore) -> Result<DockConfig, String> {
    app_state::bootstrap_pinned_apps(
        load,
        || DockImportReader.read_current_user(),
        |config| config_store.save(config),
    )
    .map(|result| result.config)
    .map_err(|error| error.to_string())
}

fn native_app_support_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())?;
    Ok(home
        .join("Library")
        .join("Application Support")
        .join("dors"))
}
