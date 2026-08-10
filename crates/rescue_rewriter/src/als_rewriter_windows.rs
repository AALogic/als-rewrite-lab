use crate::als_rewriter_io::ReplacementFailure;
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Storage::FileSystem::{
    MoveFileExW, ReplaceFileW, FILE_ATTRIBUTE_REPARSE_POINT, MOVEFILE_WRITE_THROUGH,
};

const MAX_WIN32_PATH_UNITS: usize = 260;

pub(crate) fn replace_staged_file(
    replacement: &Path,
    target: &Path,
) -> Result<(), ReplacementFailure> {
    validate_paths(replacement, target)?;
    let backup = backup_path(target);
    if backup.exists() {
        return Err(failure(
            "REWRITE_WINDOWS_BACKUP_CONFLICT",
            "Windows replacement backup already exists",
            None,
        ));
    }
    let target_wide = wide_path(target)?;
    let replacement_wide = wide_path(replacement)?;
    let backup_wide = wide_path(&backup)?;
    let replaced = unsafe {
        ReplaceFileW(
            target_wide.as_ptr(),
            replacement_wide.as_ptr(),
            backup_wide.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced != 0 {
        return remove_backup(&backup);
    }
    let replace_error = std::io::Error::last_os_error();
    preserve_previous_target(target, &backup).map_err(|rollback_error| {
        failure(
            "REWRITE_WINDOWS_REPLACE_ROLLBACK_FAILED",
            "Windows replacement failed and prior staged ALS could not be restored",
            rollback_error.raw_os_error(),
        )
    })?;
    Err(failure(
        "REWRITE_WINDOWS_REPLACE_FAILED",
        "Windows ReplaceFileW did not replace the staged ALS",
        replace_error.raw_os_error(),
    ))
}

fn validate_paths(replacement: &Path, target: &Path) -> Result<(), ReplacementFailure> {
    if replacement.parent() != target.parent() {
        return Err(failure(
            "REWRITE_WINDOWS_VOLUME_MISMATCH",
            "Replacement and staged ALS must be sibling files",
            None,
        ));
    }
    let parent = target.parent().ok_or_else(|| {
        failure(
            "REWRITE_WINDOWS_PARENT_INVALID",
            "Staged ALS has no parent directory",
            None,
        )
    })?;
    reject_reparse_ancestors(parent)?;
    wide_path(replacement)?;
    wide_path(target)?;
    Ok(())
}

fn reject_reparse_ancestors(path: &Path) -> Result<(), ReplacementFailure> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|error| {
            failure(
                "REWRITE_WINDOWS_PATH_INSPECTION_FAILED",
                "Cannot inspect staged ALS path",
                error.raw_os_error(),
            )
        })?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(failure(
                "REWRITE_WINDOWS_REPARSE_POINT_UNSUPPORTED",
                "Staged ALS path crosses a Windows reparse point",
                None,
            ));
        }
    }
    Ok(())
}

fn preserve_previous_target(target: &Path, backup: &Path) -> std::io::Result<()> {
    if target.exists() || !backup.exists() {
        return Ok(());
    }
    let backup_wide = wide_path_io(backup)?;
    let target_wide = wide_path_io(target)?;
    let moved = unsafe {
        MoveFileExW(
            backup_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn remove_backup(path: &Path) -> Result<(), ReplacementFailure> {
    fs::remove_file(path).map_err(|error| {
        failure(
            "REWRITE_WINDOWS_BACKUP_CLEANUP_FAILED",
            "Staged ALS was replaced but its temporary backup remains",
            error.raw_os_error(),
        )
    })
}

fn backup_path(target: &Path) -> PathBuf {
    let filename = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("set.als");
    target.with_file_name(format!(".{filename}.rescue-rewrite.backup"))
}

fn wide_path(path: &Path) -> Result<Vec<u16>, ReplacementFailure> {
    wide_os(path.as_os_str())
        .map_err(|message| failure("REWRITE_WINDOWS_PATH_UNSUPPORTED", &message, None))
}

fn wide_path_io(path: &Path) -> std::io::Result<Vec<u16>> {
    wide_os(path.as_os_str())
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))
}

fn wide_os(value: &OsStr) -> Result<Vec<u16>, String> {
    let mut wide: Vec<u16> = value.encode_wide().collect();
    if wide.contains(&0) {
        return Err("Windows path contains an interior NUL".to_string());
    }
    if wide.len().saturating_add(1) >= MAX_WIN32_PATH_UNITS {
        return Err("Windows Alpha does not support paths at or above MAX_PATH".to_string());
    }
    wide.push(0);
    Ok(wide)
}

fn failure(code: &'static str, message: &str, raw_os_error: Option<i32>) -> ReplacementFailure {
    let message = match raw_os_error {
        Some(value) => format!("{message}; windows_error_code={value}"),
        None => message.to_string(),
    };
    ReplacementFailure { code, message }
}
