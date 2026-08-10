use crate::{package_promoter_impl::DirectoryMoveFailure, PackagePromotionError};
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::{Component, Path, Prefix};
use windows_sys::Win32::Storage::FileSystem::{
    GetVolumeInformationW, GetVolumePathNameW, MoveFileExW, FILE_ATTRIBUTE_REPARSE_POINT,
    MOVEFILE_WRITE_THROUGH,
};

const MAX_WIN32_PATH_UNITS: usize = 260;

pub(crate) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

pub(crate) fn validate_environment(
    staging: &Path,
    target: &Path,
) -> Result<(), PackagePromotionError> {
    let target_parent = target.parent().ok_or_else(|| {
        error(
            "PROMOTION_TARGET_PARENT_INVALID",
            "Final target has no parent directory",
            target,
            None,
        )
    })?;
    require_local_drive(staging)?;
    require_local_drive(target_parent)?;
    reject_reparse_ancestors(staging)?;
    reject_reparse_ancestors(target_parent)?;
    let staging_volume = volume_root(staging)?;
    let target_volume = volume_root(target_parent)?;
    if !staging_volume.eq_ignore_ascii_case(&target_volume) {
        return Err(error(
            "PROMOTION_CROSS_DEVICE",
            "Windows promotion requires staging and target on one volume",
            target,
            None,
        ));
    }
    require_ntfs(&staging_volume, staging)?;
    Ok(())
}

pub(crate) fn move_directory_no_replace(
    source: &Path,
    target: &Path,
) -> Result<(), DirectoryMoveFailure> {
    let source_wide = wide_path(source).map_err(move_path_failure)?;
    let target_wide = wide_path(target).map_err(move_path_failure)?;
    let moved = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        let os_error = std::io::Error::last_os_error();
        return Err(DirectoryMoveFailure {
            code: "PROMOTION_WINDOWS_MOVE_FAILED",
            message: format!(
                "Windows MoveFileExW did not promote the package; windows_error_code={}",
                os_error.raw_os_error().unwrap_or_default()
            ),
        });
    }
    Ok(())
}

fn require_local_drive(path: &Path) -> Result<(), PackagePromotionError> {
    let is_local = matches!(
        path.components().next(),
        Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
    );
    if !is_local {
        return Err(error(
            "PROMOTION_WINDOWS_NONLOCAL_PATH",
            "Windows Alpha requires a local drive-letter path",
            path,
            None,
        ));
    }
    wide_path(path)
        .map_err(|message| error("PROMOTION_WINDOWS_PATH_UNSUPPORTED", &message, path, None))?;
    Ok(())
}

fn reject_reparse_ancestors(path: &Path) -> Result<(), PackagePromotionError> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|failure| {
            error(
                "PROMOTION_WINDOWS_PATH_INSPECTION_FAILED",
                "Cannot inspect Windows package path",
                ancestor,
                failure.raw_os_error(),
            )
        })?;
        if is_reparse_point(&metadata) {
            return Err(error(
                "PROMOTION_WINDOWS_REPARSE_POINT_UNSUPPORTED",
                "Windows package path crosses a reparse point",
                ancestor,
                None,
            ));
        }
    }
    Ok(())
}

fn volume_root(path: &Path) -> Result<String, PackagePromotionError> {
    let path_wide = wide_path(path)
        .map_err(|message| error("PROMOTION_WINDOWS_PATH_UNSUPPORTED", &message, path, None))?;
    let mut root = [0_u16; MAX_WIN32_PATH_UNITS];
    let ok =
        unsafe { GetVolumePathNameW(path_wide.as_ptr(), root.as_mut_ptr(), root.len() as u32) };
    if ok == 0 {
        let failure = std::io::Error::last_os_error();
        return Err(error(
            "PROMOTION_WINDOWS_VOLUME_QUERY_FAILED",
            "Cannot determine Windows volume",
            path,
            failure.raw_os_error(),
        ));
    }
    Ok(String::from_utf16_lossy(nul_terminated(&root)))
}

fn require_ntfs(volume_root: &str, path: &Path) -> Result<(), PackagePromotionError> {
    let volume_wide = wide_os(OsStr::new(volume_root))
        .map_err(|message| error("PROMOTION_WINDOWS_PATH_UNSUPPORTED", &message, path, None))?;
    let mut filesystem = [0_u16; 32];
    let ok = unsafe {
        GetVolumeInformationW(
            volume_wide.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    };
    if ok == 0 {
        let failure = std::io::Error::last_os_error();
        return Err(error(
            "PROMOTION_WINDOWS_FILESYSTEM_QUERY_FAILED",
            "Cannot determine Windows filesystem",
            path,
            failure.raw_os_error(),
        ));
    }
    let observed = String::from_utf16_lossy(nul_terminated(&filesystem));
    if !observed.eq_ignore_ascii_case("NTFS") {
        return Err(error(
            "PROMOTION_WINDOWS_FILESYSTEM_UNSUPPORTED",
            "Windows Alpha supports local NTFS only",
            path,
            None,
        ));
    }
    Ok(())
}

fn wide_path(path: &Path) -> Result<Vec<u16>, String> {
    wide_os(path.as_os_str())
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

fn nul_terminated(buffer: &[u16]) -> &[u16] {
    let length = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    &buffer[..length]
}

fn move_path_failure(message: String) -> DirectoryMoveFailure {
    DirectoryMoveFailure {
        code: "PROMOTION_WINDOWS_PATH_UNSUPPORTED",
        message,
    }
}

fn error(
    code: &str,
    message: &str,
    path: &Path,
    raw_os_error: Option<i32>,
) -> PackagePromotionError {
    let message = match raw_os_error {
        Some(value) => format!("{message}; windows_error_code={value}"),
        None => message.to_string(),
    };
    crate::package_promoter_result::error(code, message, Some(path))
}
