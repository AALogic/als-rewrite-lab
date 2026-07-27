use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct ArtifactWrite {
    pub sha256: String,
    pub size: u64,
    pub status: String,
}

pub(crate) struct ArtifactWriteFailure {
    pub code: &'static str,
    pub message: String,
    pub path: PathBuf,
}

pub(crate) fn serialize_json<T: serde::Serialize>(
    value: &T,
) -> Result<Vec<u8>, ArtifactWriteFailure> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| ArtifactWriteFailure {
        code: "MANIFEST_SERIALIZE_FAILED",
        message: format!("Cannot serialize JSON artifact: {error}"),
        path: PathBuf::new(),
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn write_json_noclobber(
    target: &Path,
    bytes: &[u8],
    create_parent: bool,
) -> Result<ArtifactWrite, ArtifactWriteFailure> {
    prepare_parent(target, create_parent)?;
    if target.exists() {
        return verify_existing(target, bytes);
    }
    let temp = temporary_path(target);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| {
            failure(
                "MANIFEST_TEMP_CREATE_FAILED",
                format!("Cannot create temporary JSON artifact: {error}"),
                &temp,
            )
        })?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temp);
        return Err(failure(
            "MANIFEST_TEMP_WRITE_FAILED",
            format!("Cannot persist temporary JSON artifact: {error}"),
            &temp,
        ));
    }
    if let Err(error) = fs::hard_link(&temp, target) {
        let _ = fs::remove_file(&temp);
        if target.exists() {
            return verify_existing(target, bytes);
        }
        return Err(failure(
            "MANIFEST_ATOMIC_CREATE_FAILED",
            format!("Cannot atomically create JSON artifact: {error}"),
            target,
        ));
    }
    if let Err(error) = fs::remove_file(&temp) {
        return Err(failure(
            "MANIFEST_TEMP_CLEANUP_FAILED",
            format!("JSON artifact exists but temporary link remains: {error}"),
            &temp,
        ));
    }
    sync_parent(target)?;
    Ok(ArtifactWrite {
        sha256: digest(bytes),
        size: bytes.len() as u64,
        status: "written_and_verified".to_string(),
    })
}

fn prepare_parent(target: &Path, create_parent: bool) -> Result<(), ArtifactWriteFailure> {
    let parent = target.parent().ok_or_else(|| {
        failure(
            "MANIFEST_PARENT_MISSING",
            "JSON artifact has no parent directory".to_string(),
            target,
        )
    })?;
    if create_parent {
        fs::create_dir_all(parent).map_err(|error| {
            failure(
                "MANIFEST_PARENT_CREATE_FAILED",
                format!("Cannot create manifest parent directory: {error}"),
                parent,
            )
        })?;
    }
    let metadata = fs::symlink_metadata(parent).map_err(|error| {
        failure(
            "MANIFEST_PARENT_INVALID",
            format!("Cannot inspect manifest parent: {error}"),
            parent,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(failure(
            "MANIFEST_PARENT_INVALID",
            "Manifest parent must be a non-symlink directory".to_string(),
            parent,
        ));
    }
    Ok(())
}

fn verify_existing(target: &Path, bytes: &[u8]) -> Result<ArtifactWrite, ArtifactWriteFailure> {
    let metadata = fs::symlink_metadata(target).map_err(|error| {
        failure(
            "MANIFEST_EXISTING_READ_FAILED",
            format!("Cannot inspect existing JSON artifact: {error}"),
            target,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(failure(
            "MANIFEST_TARGET_CONFLICT",
            "Existing manifest target is not a regular file".to_string(),
            target,
        ));
    }
    let existing = fs::read(target).map_err(|error| {
        failure(
            "MANIFEST_EXISTING_READ_FAILED",
            format!("Cannot read existing JSON artifact: {error}"),
            target,
        )
    })?;
    if existing != bytes {
        return Err(failure(
            "MANIFEST_TARGET_CONFLICT",
            "Existing JSON artifact has different content".to_string(),
            target,
        ));
    }
    Ok(ArtifactWrite {
        sha256: digest(bytes),
        size: bytes.len() as u64,
        status: "already_present_verified".to_string(),
    })
}

#[cfg(unix)]
fn sync_parent(target: &Path) -> Result<(), ArtifactWriteFailure> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            failure(
                "MANIFEST_PARENT_SYNC_FAILED",
                format!("Cannot sync JSON artifact parent: {error}"),
                parent,
            )
        })
}

#[cfg(not(unix))]
fn sync_parent(_target: &Path) -> Result<(), ArtifactWriteFailure> {
    Ok(())
}

fn temporary_path(target: &Path) -> PathBuf {
    let filename = target
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("manifest.json");
    target.with_file_name(format!(".{filename}.rescue-write.tmp"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn failure(code: &'static str, message: String, path: &Path) -> ArtifactWriteFailure {
    ArtifactWriteFailure {
        code,
        message,
        path: path.to_path_buf(),
    }
}
