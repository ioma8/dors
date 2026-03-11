use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use plist::Value;

static ICON_CACHE: OnceLock<Mutex<std::collections::HashMap<PathBuf, Option<String>>>> =
    OnceLock::new();

pub fn resolve_icon_path_from_bundle(app_path: &Path) -> Option<PathBuf> {
    let info_plist = app_path.join("Contents/Info.plist");
    let plist = Value::from_file(&info_plist).ok()?;
    let icon_name = plist
        .as_dictionary()?
        .get("CFBundleIconFile")?
        .as_string()?
        .trim();
    if icon_name.is_empty() {
        return None;
    }

    let resource_name = if icon_name.ends_with(".icns") {
        icon_name.to_string()
    } else {
        format!("{icon_name}.icns")
    };
    let resource_path = PathBuf::from("Contents/Resources").join(resource_name);

    app_path
        .join(&resource_path)
        .is_file()
        .then_some(resource_path)
}

pub fn load_icon_data_url(app_path: &Path) -> Option<String> {
    load_icon_data_url_with(app_path, convert_icns_to_png)
}

pub fn load_icon_data_url_with<Convert>(app_path: &Path, convert: Convert) -> Option<String>
where
    Convert: Fn(&Path) -> Result<Vec<u8>, ()>,
{
    let cache = icon_cache();
    if let Some(cached) = cache
        .lock()
        .ok()
        .and_then(|entries| entries.get(app_path).cloned())
    {
        return cached;
    }

    let icon_path = app_path.join(resolve_icon_path_from_bundle(app_path)?);
    let resolved = convert(&icon_path).ok().map(|png_bytes| {
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png_bytes)
        )
    });

    if let Ok(mut entries) = cache.lock() {
        entries.insert(app_path.to_path_buf(), resolved.clone());
    }

    resolved
}

fn convert_icns_to_png(icon_path: &Path) -> Result<Vec<u8>, ()> {
    let output_path = temporary_png_path();
    let status = Command::new("sips")
        .args(["-s", "format", "png"])
        .arg(icon_path)
        .args(["--out"])
        .arg(&output_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| ())?;
    if !status.success() {
        let _ = fs::remove_file(&output_path);
        return Err(());
    }

    let png_bytes = fs::read(&output_path).map_err(|_| ())?;
    let _ = fs::remove_file(output_path);
    Ok(png_bytes)
}

fn temporary_png_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("dors-icon-{timestamp}.png"))
}

fn icon_cache() -> &'static Mutex<std::collections::HashMap<PathBuf, Option<String>>> {
    ICON_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

pub fn clear_icon_cache_for_tests() {
    match icon_cache().lock() {
        Ok(mut entries) => entries.clear(),
        Err(poisoned) => poisoned.into_inner().clear(),
    }
}
