use crate::PackagePromotionError;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

pub(crate) fn inspect_regular_file(
    path: &Path,
    verification_method: &str,
) -> Result<(Option<String>, u64), PackagePromotionError> {
    match verification_method {
        rescue_packaging::VERIFY_SHA256_AND_SIZE => {
            hash_regular_file(path).map(|(hash, size)| (Some(hash), size))
        }
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE => {
            regular_file_size(path).map(|size| (None, size))
        }
        _ => Err(error(
            "PROMOTION_VERIFICATION_POLICY_UNSUPPORTED",
            "Package file verification policy is unsupported",
            path,
        )),
    }
}

pub(crate) fn hash_regular_file(path: &Path) -> Result<(String, u64), PackagePromotionError> {
    require_regular_file(path)?;
    let mut file = File::open(path)
        .map_err(|_| error("PROMOTION_FILE_READ_FAILED", "Cannot open file", path))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| error("PROMOTION_FILE_READ_FAILED", "Cannot read file", path))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

fn regular_file_size(path: &Path) -> Result<u64, PackagePromotionError> {
    require_regular_file(path).map(|metadata| metadata.len())
}

fn require_regular_file(path: &Path) -> Result<fs::Metadata, PackagePromotionError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| error("PROMOTION_FILE_UNAVAILABLE", "Cannot inspect file", path))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(error(
            "PROMOTION_FILE_NOT_REGULAR",
            "Expected path is not a regular non-symlink file",
            path,
        ));
    }
    Ok(metadata)
}

fn error(code: &str, message: &str, path: &Path) -> PackagePromotionError {
    crate::package_promoter_result::error(code, message, Some(path))
}
