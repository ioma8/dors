use std::sync::Mutex;

use thiserror::Error;

use crate::adapters::dock_import::{DockImportError, DockImportResult};
use crate::config::{ConfigLoad, ConfigStoreError, DockConfig};
use crate::domain::{DockItemView, RunningApp};
use crate::services::dock_state::build_dock_items;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapResult {
    pub config: DockConfig,
    pub imported_on_first_run: bool,
}

#[derive(Debug, Default)]
pub struct AppState {
    dock_items: Mutex<Vec<DockItemView>>,
    pinned_config: DockConfig,
}

#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("failed to import macOS dock state")]
    Import(#[source] DockImportError),
    #[error("failed to persist imported config")]
    Save(#[source] ConfigStoreError),
}

pub fn bootstrap_pinned_apps<Import, Save>(
    load: ConfigLoad,
    import: Import,
    save: Save,
) -> Result<BootstrapResult, BootstrapError>
where
    Import: FnOnce() -> Result<DockImportResult, DockImportError>,
    Save: FnOnce(&DockConfig) -> Result<(), ConfigStoreError>,
{
    match load {
        ConfigLoad::Loaded(config) => Ok(BootstrapResult {
            config,
            imported_on_first_run: false,
        }),
        ConfigLoad::Missing => {
            let imported = import().map_err(BootstrapError::Import)?;
            let config = DockConfig {
                pinned_apps: imported.apps,
            };
            save(&config).map_err(BootstrapError::Save)?;
            Ok(BootstrapResult {
                config,
                imported_on_first_run: true,
            })
        }
    }
}

impl AppState {
    pub fn new(pinned_config: DockConfig) -> Self {
        Self {
            dock_items: Mutex::new(Vec::new()),
            pinned_config,
        }
    }

    pub fn dock_items(&self) -> Result<Vec<DockItemView>, String> {
        self.dock_items
            .lock()
            .map(|items| items.clone())
            .map_err(|_| "dock state lock poisoned".to_string())
    }

    pub fn replace_dock_items(&self, dock_items: Vec<DockItemView>) -> Result<(), String> {
        self.dock_items
            .lock()
            .map(|mut items| {
                *items = dock_items;
            })
            .map_err(|_| "dock state lock poisoned".to_string())
    }

    pub fn refresh_with_running_apps(
        &self,
        running_apps: Vec<RunningApp>,
    ) -> Result<Vec<DockItemView>, String> {
        let dock_items = build_dock_items(&self.pinned_config, &running_apps);
        self.replace_dock_items(dock_items.clone())?;
        Ok(dock_items)
    }
}
