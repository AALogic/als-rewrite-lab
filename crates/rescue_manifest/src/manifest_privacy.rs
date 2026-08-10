use crate::PackageManifest;
use serde_json::Value;
use std::path::{Component, Path};

pub(crate) fn validate_portable_manifest(manifest: &PackageManifest) -> Result<(), String> {
    for directory in &manifest.directories {
        if !safe_relative(&directory.relative_path) {
            return Err("Package manifest contains an unsafe directory path".to_string());
        }
    }
    for file in &manifest.files {
        if !safe_relative(&file.relative_path) {
            return Err("Package manifest contains an unsafe file path".to_string());
        }
    }
    let value = serde_json::to_value(manifest)
        .map_err(|error| format!("Cannot inspect package manifest privacy: {error}"))?;
    inspect_value(&value)
}

fn inspect_value(value: &Value) -> Result<(), String> {
    match value {
        Value::String(text) if looks_like_absolute_path(text) => {
            Err("Package manifest contains an absolute local path".to_string())
        }
        Value::Array(values) => {
            for item in values {
                inspect_value(item)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            for item in values.values() {
                inspect_value(item)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn looks_like_absolute_path(value: &str) -> bool {
    if Path::new(value).is_absolute() || value.starts_with("\\\\") {
        return true;
    }
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
