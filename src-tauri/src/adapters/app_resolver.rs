use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppResolverRecord {
    pub bundle_id: Option<String>,
    pub display_name: Option<String>,
    pub path: PathBuf,
    pub icon_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppMetadata {
    pub bundle_id: Option<String>,
    pub display_name: String,
    pub path: PathBuf,
    pub icon_path: Option<PathBuf>,
}

pub fn resolve_app_metadata(record: AppResolverRecord) -> AppMetadata {
    let path = normalize_app_path(record.path);
    let display_name = record
        .display_name
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| derive_display_name(&path));

    AppMetadata {
        bundle_id: record.bundle_id,
        display_name,
        path,
        icon_path: record.icon_path,
    }
}

fn normalize_app_path(path: PathBuf) -> PathBuf {
    let path_string = path.to_string_lossy();
    let trimmed = path_string.trim_end_matches('/');
    PathBuf::from(trimmed)
}

fn derive_display_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Unknown App")
        .to_string()
}
