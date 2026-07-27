use sha2::{Digest, Sha256};
use std::fs::{self, File, Metadata};
use std::io::{BufReader, Read};
use std::path::Path;

pub(crate) struct HashObservation {
    pub digest: String,
    pub file_size: u64,
}

pub(crate) struct HashFailure {
    pub code: &'static str,
    pub message: &'static str,
}

pub(crate) fn hash_stable_file(path: &Path) -> Result<HashObservation, HashFailure> {
    let before = fs::symlink_metadata(path).map_err(|_| {
        failure(
            "INVENTORY_FILE_METADATA_FAILED",
            "File metadata could not be read before hashing",
        )
    })?;
    if before.file_type().is_symlink() || !before.is_file() {
        return Err(failure(
            "INVENTORY_FILE_CHANGED",
            "File entry changed before hashing",
        ));
    }
    let file = File::open(path).map_err(|_| {
        failure(
            "INVENTORY_FILE_OPEN_FAILED",
            "File could not be opened for hashing",
        )
    })?;
    let opened = file.metadata().map_err(|_| {
        failure(
            "INVENTORY_FILE_METADATA_FAILED",
            "Opened file metadata could not be read",
        )
    })?;
    if !same_file_identity(&before, &opened) {
        return Err(failure(
            "INVENTORY_FILE_CHANGED",
            "File identity changed while opening",
        ));
    }

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(|_| {
            failure(
                "INVENTORY_FILE_READ_FAILED",
                "File could not be read completely for hashing",
            )
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let after_handle = reader.get_ref().metadata().map_err(|_| {
        failure(
            "INVENTORY_FILE_METADATA_FAILED",
            "Opened file metadata could not be read after hashing",
        )
    })?;
    let after_path = fs::symlink_metadata(path)
        .map_err(|_| failure("INVENTORY_FILE_CHANGED", "File path changed during hashing"))?;
    if !same_stable_snapshot(&before, &after_handle) || !same_file_identity(&before, &after_path) {
        return Err(failure(
            "INVENTORY_FILE_CHANGED",
            "File changed during hashing and was excluded",
        ));
    }

    Ok(HashObservation {
        digest: format!("{:x}", hasher.finalize()),
        file_size: after_handle.len(),
    })
}

fn same_stable_snapshot(left: &Metadata, right: &Metadata) -> bool {
    same_file_identity(left, right)
        && left.len() == right.len()
        && left.modified().ok() == right.modified().ok()
}

#[cfg(unix)]
fn same_file_identity(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file_identity(left: &Metadata, right: &Metadata) -> bool {
    left.len() == right.len() && left.modified().ok() == right.modified().ok()
}

fn failure(code: &'static str, message: &'static str) -> HashFailure {
    HashFailure { code, message }
}
