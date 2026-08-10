use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const PREFERENCES_SCHEMA_VERSION: &str = "0.1";
const PREFERENCES_FILE_NAME: &str = "courier-preferences.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierPreferences {
    pub schema_version: String,
    pub output_parent: PathBuf,
    pub local_delivery_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CourierPreferencesError {
    pub(crate) error_code: String,
    pub(crate) stage: String,
    pub(crate) message: String,
}

pub(crate) fn load_for_app(app: &AppHandle) -> Result<CourierPreferences, CourierPreferencesError> {
    let path = preferences_path(app)?;
    if path.exists() {
        read_preferences(&path)
    } else {
        default_preferences(app)
    }
}

pub(crate) fn save_for_app(
    app: &AppHandle,
    preferences: &CourierPreferences,
) -> Result<(), CourierPreferencesError> {
    validate_preferences(preferences)?;
    write_preferences_atomic(&preferences_path(app)?, preferences)
}

fn default_preferences(app: &AppHandle) -> Result<CourierPreferences, CourierPreferencesError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|_| settings_error("defaults"))?;
    Ok(CourierPreferences {
        schema_version: PREFERENCES_SCHEMA_VERSION.to_string(),
        output_parent: home.join("Downloads").join("sexy_testy"),
        local_delivery_root: None,
    })
}

fn preferences_path(app: &AppHandle) -> Result<PathBuf, CourierPreferencesError> {
    app.path()
        .app_config_dir()
        .map(|root| root.join(PREFERENCES_FILE_NAME))
        .map_err(|_| settings_error("path"))
}

fn read_preferences(path: &Path) -> Result<CourierPreferences, CourierPreferencesError> {
    let bytes = std::fs::read(path).map_err(|_| settings_error("read"))?;
    let preferences = serde_json::from_slice(&bytes).map_err(|_| settings_error("parse"))?;
    validate_preferences(&preferences)?;
    Ok(preferences)
}

fn write_preferences_atomic(
    path: &Path,
    preferences: &CourierPreferences,
) -> Result<(), CourierPreferencesError> {
    let parent = path.parent().ok_or_else(|| settings_error("write"))?;
    std::fs::create_dir_all(parent).map_err(|_| settings_error("write"))?;
    let temporary = parent.join(format!(
        ".{PREFERENCES_FILE_NAME}.tmp-{}",
        std::process::id()
    ));
    let bytes = serde_json::to_vec_pretty(preferences).map_err(|_| settings_error("serialize"))?;
    let write_result = (|| {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = options
            .open(&temporary)
            .map_err(|_| settings_error("write"))?;
        std::io::Write::write_all(&mut file, &bytes).map_err(|_| settings_error("write"))?;
        file.sync_all().map_err(|_| settings_error("write"))?;
        replace_file(&temporary, path)
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    write_result
}

fn validate_preferences(preferences: &CourierPreferences) -> Result<(), CourierPreferencesError> {
    let delivery_valid = preferences
        .local_delivery_root
        .as_ref()
        .is_none_or(|path| path.is_absolute());
    if preferences.schema_version != PREFERENCES_SCHEMA_VERSION
        || !preferences.output_parent.is_absolute()
        || !delivery_valid
    {
        return Err(settings_error("validate"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(source: &Path, target: &Path) -> Result<(), CourierPreferencesError> {
    std::fs::rename(source, target).map_err(|_| settings_error("replace"))
}

#[cfg(windows)]
fn replace_file(source: &Path, target: &Path) -> Result<(), CourierPreferencesError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let source = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // SAFETY: both pointers reference NUL-terminated UTF-16 buffers for this call.
    let result = unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if result == 0 {
        Err(settings_error("replace"))
    } else {
        Ok(())
    }
}

fn settings_error(stage: &str) -> CourierPreferencesError {
    CourierPreferencesError {
        error_code: "DELIVERY_SETTINGS_INVALID".to_string(),
        stage: stage.to_string(),
        message: "Courier preferences are unavailable or invalid.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_write_is_atomic_and_versioned() {
        let temporary = tempfile::tempdir();
        assert!(temporary.is_ok());
        let Some(temporary) = temporary.ok() else {
            return;
        };
        let path = temporary.path().join(PREFERENCES_FILE_NAME);
        let first = CourierPreferences {
            schema_version: "0.1".to_string(),
            output_parent: temporary.path().join("output"),
            local_delivery_root: None,
        };
        assert!(write_preferences_atomic(&path, &first).is_ok());
        let second = CourierPreferences {
            local_delivery_root: Some(temporary.path().join("delivery")),
            ..first
        };
        assert!(write_preferences_atomic(&path, &second).is_ok());
        assert_eq!(read_preferences(&path), Ok(second));
        let leftovers = std::fs::read_dir(temporary.path())
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
            .count();
        assert_eq!(leftovers, 0);
    }
}
