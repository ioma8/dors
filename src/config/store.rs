use std::fs;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::DockConfig;

const CONFIG_FILE_NAME: &str = "dock-config.json";
const TEMP_FILE_NAME: &str = "dock-config.json.tmp";

#[derive(Debug, Eq, PartialEq)]
pub enum ConfigLoad {
    Missing,
    Loaded(DockConfig),
}

#[derive(Debug)]
pub struct ConfigStore {
    app_support_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigStoreError {
    #[error("failed to access config directory")]
    CreateDir(#[source] io::Error),
    #[error("failed to read config file")]
    Read(#[source] io::Error),
    #[error("failed to write config file")]
    Write(#[source] io::Error),
    #[error("failed to persist config file")]
    Persist(#[source] io::Error),
    #[error("failed to serialize config")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to deserialize config")]
    Deserialize(#[source] serde_json::Error),
}

impl ConfigStore {
    pub fn new(app_support_dir: PathBuf) -> Self {
        Self { app_support_dir }
    }

    pub fn load(&self) -> Result<ConfigLoad, ConfigStoreError> {
        let config_path = self.config_path();
        if !config_path.exists() {
            return Ok(ConfigLoad::Missing);
        }

        let contents = fs::read_to_string(config_path).map_err(ConfigStoreError::Read)?;
        let config = serde_json::from_str(&contents).map_err(ConfigStoreError::Deserialize)?;
        Ok(ConfigLoad::Loaded(config))
    }

    pub fn save(&self, config: &DockConfig) -> Result<(), ConfigStoreError> {
        fs::create_dir_all(&self.app_support_dir).map_err(ConfigStoreError::CreateDir)?;

        let contents = serde_json::to_vec_pretty(config).map_err(ConfigStoreError::Serialize)?;
        let temp_path = self.temp_path();
        fs::write(&temp_path, contents).map_err(ConfigStoreError::Write)?;
        fs::rename(temp_path, self.config_path()).map_err(ConfigStoreError::Persist)?;
        Ok(())
    }

    pub fn config_path(&self) -> PathBuf {
        self.app_support_dir.join(CONFIG_FILE_NAME)
    }

    fn temp_path(&self) -> PathBuf {
        self.app_support_dir.join(TEMP_FILE_NAME)
    }
}
