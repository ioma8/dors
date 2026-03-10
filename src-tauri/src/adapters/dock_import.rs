use std::env;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use plist::Value;
use thiserror::Error;

use crate::domain::{AppIdentity, PinnedApp};

const DOCK_PLIST_RELATIVE_PATH: &str = "Library/Preferences/com.apple.dock.plist";

#[derive(Debug, Default)]
pub struct DockImportReader;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockImportResult {
    pub apps: Vec<PinnedApp>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum DockImportError {
    #[error("failed to read dock preferences")]
    Read(#[source] std::io::Error),
    #[error("failed to parse dock preferences")]
    Parse(#[source] plist::Error),
    #[error("dock preferences root must be a dictionary")]
    InvalidRoot,
}

impl DockImportReader {
    pub fn read_current_user(&self) -> Result<DockImportResult, DockImportError> {
        let home_dir = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(DockImportError::InvalidRoot)?;
        let plist_bytes =
            fs::read(home_dir.join(DOCK_PLIST_RELATIVE_PATH)).map_err(DockImportError::Read)?;
        self.parse_plist_bytes(&plist_bytes)
    }

    pub fn parse_plist_bytes(
        &self,
        plist_bytes: &[u8],
    ) -> Result<DockImportResult, DockImportError> {
        let plist = Value::from_reader(Cursor::new(plist_bytes)).map_err(DockImportError::Parse)?;
        let root = plist.as_dictionary().ok_or(DockImportError::InvalidRoot)?;

        let items = root
            .get("persistent-apps")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut apps = Vec::new();
        let mut warnings = Vec::new();

        for item in items {
            match parse_pinned_app(&item) {
                Some(app) => apps.push(app),
                None => warnings.push("skipped broken entry from Dock preferences".to_string()),
            }
        }

        Ok(DockImportResult { apps, warnings })
    }
}

fn parse_pinned_app(item: &Value) -> Option<PinnedApp> {
    let tile_data = item.as_dictionary()?.get("tile-data")?.as_dictionary()?;
    let path = tile_data
        .get("file-data")?
        .as_dictionary()?
        .get("_CFURLString")?
        .as_string()?;

    let display_name = tile_data.get("file-label")?.as_string()?.to_string();
    let bundle_id = tile_data
        .get("bundle-identifier")
        .and_then(Value::as_string)
        .map(ToString::to_string);

    Some(PinnedApp {
        identity: AppIdentity {
            bundle_id,
            path: PathBuf::from(path),
        },
        display_name,
    })
}
