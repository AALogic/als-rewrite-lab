use crate::project_catalog_store_io::ReplacementFailure;
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, ReplaceFileW, MOVEFILE_WRITE_THROUGH};

const MAX_WIN32_PATH_UNITS: usize = 260;

pub(crate) fn replace_file(temp: &Path, target: &Path) -> Result<(), ReplacementFailure> {
    if temp.parent() != target.parent() {
        return Err(failure(
            "Private catalog replacement files must be siblings",
            None,
        ));
    }
    let backup = backup_path(target);
    if backup.exists() {
        return Err(failure(
            "Private catalog replacement backup already exists",
            None,
        ));
    }
    let target_wide = wide_path(target)?;
    let temp_wide = wide_path(temp)?;
    let backup_wide = wide_path(&backup)?;
    let replaced = unsafe {
        ReplaceFileW(
            target_wide.as_ptr(),
            temp_wide.as_ptr(),
            backup_wide.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced != 0 {
        return fs::remove_file(&backup).map_err(|error| {
            failure(
                "Private catalog was replaced but its backup remains",
                error.raw_os_error(),
            )
        });
    }
    let replace_error = std::io::Error::last_os_error();
    restore_backup(target, &backup).map_err(|error| {
        failure(
            "Private catalog replacement failed and rollback also failed",
            error.raw_os_error(),
        )
    })?;
    Err(failure(
        "Windows could not atomically replace the private catalog",
        replace_error.raw_os_error(),
    ))
}

fn restore_backup(target: &Path, backup: &Path) -> std::io::Result<()> {
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
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn backup_path(target: &Path) -> PathBuf {
    target.with_file_name(".ableton-project-rescue.catalog.backup")
}

fn wide_path(path: &Path) -> Result<Vec<u16>, ReplacementFailure> {
    wide_os(path.as_os_str()).map_err(|message| failure(&message, None))
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
        return Err(
            "Windows Alpha does not support catalog paths at or above MAX_PATH".to_string(),
        );
    }
    wide.push(0);
    Ok(wide)
}

fn failure(message: &str, raw_os_error: Option<i32>) -> ReplacementFailure {
    ReplacementFailure {
        code: "CATALOG_STORE_ATOMIC_REPLACE_FAILED",
        message: match raw_os_error {
            Some(code) => format!("{message}; windows_error_code={code}"),
            None => message.to_string(),
        },
    }
}
