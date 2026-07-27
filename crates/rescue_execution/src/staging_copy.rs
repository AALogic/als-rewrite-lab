use rescue_packaging::CopyOperation;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub(crate) struct CopyOutcome {
    pub digest: String,
    pub size: u64,
}

pub(crate) struct CopyFailure {
    pub code: &'static str,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub(crate) fn copy_verified(
    staging_root: &Path,
    operation: &CopyOperation,
    index: usize,
) -> Result<CopyOutcome, CopyFailure> {
    let source_metadata = fs::symlink_metadata(&operation.source_path).map_err(|error| {
        failure(
            "COPY_SOURCE_UNAVAILABLE",
            format!("Cannot inspect copy source: {error}"),
            Some(operation.source_path.clone()),
        )
    })?;
    if source_metadata.file_type().is_symlink() || !source_metadata.file_type().is_file() {
        return Err(failure(
            "COPY_SOURCE_NOT_REGULAR_FILE",
            "Copy source must be a regular non-symlink file".to_string(),
            Some(operation.source_path.clone()),
        ));
    }

    let target = staging_root.join(&operation.target_relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            failure(
                "COPY_TARGET_CREATE_FAILED",
                format!("Cannot create staging directory: {error}"),
                Some(parent.to_path_buf()),
            )
        })?;
    }
    if target.exists() {
        return Err(failure(
            "COPY_TARGET_EXISTS",
            "Staging target already exists".to_string(),
            Some(target),
        ));
    }

    let temp = temporary_path(&target, index);
    let outcome = stream_copy(&operation.source_path, &temp)?;
    if outcome.size != operation.expected_source_size {
        remove_own_temp(&temp);
        return Err(failure(
            "COPY_SIZE_MISMATCH",
            format!(
                "Expected {} bytes but copied {} bytes",
                operation.expected_source_size, outcome.size
            ),
            Some(operation.source_path.clone()),
        ));
    }
    if outcome.digest != operation.expected_source_sha256 {
        remove_own_temp(&temp);
        return Err(failure(
            "COPY_HASH_MISMATCH",
            "Copied bytes do not match the planned SHA-256".to_string(),
            Some(operation.source_path.clone()),
        ));
    }
    fs::rename(&temp, &target).map_err(|error| {
        remove_own_temp(&temp);
        failure(
            "COPY_PROMOTION_FAILED",
            format!("Cannot promote verified temporary copy: {error}"),
            Some(target),
        )
    })?;
    Ok(outcome)
}

fn stream_copy(source: &Path, temp: &Path) -> Result<CopyOutcome, CopyFailure> {
    let mut input = File::open(source).map_err(|error| {
        failure(
            "COPY_SOURCE_OPEN_FAILED",
            format!("Cannot open copy source: {error}"),
            Some(source.to_path_buf()),
        )
    })?;
    let opened_metadata = input.metadata().map_err(|error| {
        failure(
            "COPY_SOURCE_METADATA_FAILED",
            format!("Cannot inspect opened copy source: {error}"),
            Some(source.to_path_buf()),
        )
    })?;
    if !opened_metadata.is_file() {
        return Err(failure(
            "COPY_SOURCE_NOT_REGULAR_FILE",
            "Opened copy source is not a regular file".to_string(),
            Some(source.to_path_buf()),
        ));
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp)
        .map_err(|error| {
            failure(
                "COPY_TEMP_CREATE_FAILED",
                format!("Cannot create temporary copy: {error}"),
                Some(temp.to_path_buf()),
            )
        })?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input.read(&mut buffer).map_err(|error| {
            remove_own_temp(temp);
            failure(
                "COPY_READ_FAILED",
                format!("Cannot read copy source: {error}"),
                Some(source.to_path_buf()),
            )
        })?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read]).map_err(|error| {
            remove_own_temp(temp);
            failure(
                "COPY_WRITE_FAILED",
                format!("Cannot write temporary copy: {error}"),
                Some(temp.to_path_buf()),
            )
        })?;
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    output.sync_all().map_err(|error| {
        remove_own_temp(temp);
        failure(
            "COPY_SYNC_FAILED",
            format!("Cannot sync temporary copy: {error}"),
            Some(temp.to_path_buf()),
        )
    })?;
    Ok(CopyOutcome {
        digest: format!("{:x}", hasher.finalize()),
        size,
    })
}

fn temporary_path(target: &Path, index: usize) -> PathBuf {
    let filename = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("copy");
    target.with_file_name(format!(".{filename}.rescue-copy-{index:06}.tmp"))
}

fn remove_own_temp(path: &Path) {
    let _ = fs::remove_file(path);
}

fn failure(code: &'static str, message: String, path: Option<PathBuf>) -> CopyFailure {
    CopyFailure {
        code,
        message,
        path,
    }
}
