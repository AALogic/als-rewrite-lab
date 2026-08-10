use std::path::{Path, PathBuf};
use tauri::AppHandle;

pub(crate) fn route_finder_drop(
    app: &AppHandle,
    paths: &[PathBuf],
) -> Result<crate::courier_commands::CourierIntakeResult, rescue_application::DesktopApplicationError>
{
    let supported = paths
        .iter()
        .filter(|path| is_supported_als_path(path))
        .cloned()
        .collect::<Vec<_>>();
    crate::courier_commands::intake_native_paths(app, &supported)
}

pub(crate) fn is_supported_als_path(path: &Path) -> bool {
    validate_als_path(path).is_ok()
}

pub(crate) fn validate_als_path(path: &Path) -> Result<String, &'static str> {
    if !path.is_absolute() {
        return Err("QUICK_COPY_SOURCE_NOT_REGULAR_FILE");
    }
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| "QUICK_COPY_SOURCE_NOT_REGULAR_FILE")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("QUICK_COPY_SOURCE_NOT_REGULAR_FILE");
    }
    let is_als = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("als"));
    if !is_als {
        return Err("QUICK_COPY_SOURCE_NOT_ALS");
    }
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.trim().is_empty())
        .ok_or("QUICK_COPY_SOURCE_NAME_INVALID")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn finder_intake_accepts_only_regular_als_files() {
        let Ok(directory) = tempfile::tempdir() else {
            panic!("fixture directory should be available");
        };
        let als = directory.path().join("drop.ALS");
        let text = directory.path().join("drop.txt");
        assert!(File::create(&als).is_ok());
        assert!(File::create(&text).is_ok());
        assert!(is_supported_als_path(&als));
        assert!(!is_supported_als_path(&text));
        assert!(!is_supported_als_path(directory.path()));
    }

    #[cfg(unix)]
    #[test]
    fn finder_intake_rejects_symlinked_als_files() {
        use std::os::unix::fs::symlink;

        let Ok(directory) = tempfile::tempdir() else {
            panic!("fixture directory should be available");
        };
        let target = directory.path().join("target.als");
        let link = directory.path().join("link.als");
        assert!(File::create(&target).is_ok());
        assert!(symlink(&target, &link).is_ok());
        assert!(!is_supported_als_path(&link));
    }
}
