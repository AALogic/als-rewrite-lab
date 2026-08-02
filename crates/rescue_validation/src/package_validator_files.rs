use crate::{FileValidationRecord, PackageValidationError};
use rescue_packaging::{CopyOperation, PackagePlan};
use rescue_rewriter::ALSRewriteResult;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub(crate) struct FileValidationOutcome {
    pub records: Vec<FileValidationRecord>,
    pub errors: Vec<PackageValidationError>,
}

pub(crate) fn validate_files(
    staging_root: &Path,
    plan: &PackagePlan,
    rewrite: &ALSRewriteResult,
) -> FileValidationOutcome {
    let mut errors = Vec::new();
    validate_tree_membership(staging_root, &plan.copy_operations, &mut errors);
    let mut records = Vec::new();
    for operation in &plan.copy_operations {
        let (expected_hash, expected_size, verification_method) =
            expected_file_state(operation, rewrite);
        records.push(validate_file(
            staging_root,
            operation,
            expected_hash,
            expected_size,
            verification_method,
            &mut errors,
        ));
    }
    FileValidationOutcome { records, errors }
}

fn expected_file_state<'a>(
    operation: &'a CopyOperation,
    rewrite: &'a ALSRewriteResult,
) -> (Option<&'a str>, Option<u64>, &'a str) {
    if operation.operation_kind == "copy_als" && rewrite.rewrite_status == "rewrite_complete" {
        (
            rewrite.rewritten_staged_als_hash.as_deref(),
            None,
            rescue_packaging::VERIFY_SHA256_AND_SIZE,
        )
    } else {
        (
            operation.expected_source_sha256.as_deref(),
            Some(operation.expected_source_size),
            operation.verification_policy.as_str(),
        )
    }
}

fn validate_file(
    staging_root: &Path,
    operation: &CopyOperation,
    expected_hash: Option<&str>,
    expected_size: Option<u64>,
    verification_method: &str,
    errors: &mut Vec<PackageValidationError>,
) -> FileValidationRecord {
    let path = staging_root.join(&operation.target_relative_path);
    let observed = inspect_regular_file(&path, verification_method);
    let (observed_hash, observed_size, status) = match observed {
        Ok((hash, size))
            if hash.as_deref() == expected_hash
                && expected_size.is_none_or(|value| value == size) =>
        {
            (hash, Some(size), "verified")
        }
        Ok((hash, size)) => {
            let code = if hash.as_deref() != expected_hash {
                "VALIDATION_FILE_HASH_MISMATCH"
            } else {
                "VALIDATION_FILE_SIZE_MISMATCH"
            };
            errors.push(crate::package_validator_result::error(
                code,
                "Staged file does not match the approved package state",
                Some(&operation.operation_id),
                Some(&path),
            ));
            (hash, Some(size), "mismatch")
        }
        Err(error) => {
            errors.push(crate::package_validator_result::error(
                error.code,
                error.message,
                Some(&operation.operation_id),
                Some(&path),
            ));
            (None, None, "missing_or_invalid")
        }
    };
    FileValidationRecord {
        operation_id: operation.operation_id.clone(),
        target_relative_path: operation.target_relative_path.clone(),
        expected_sha256: expected_hash.map(str::to_string),
        observed_sha256: observed_hash,
        expected_size,
        observed_size,
        verification_method: verification_method.to_string(),
        file_status: status.to_string(),
    }
}

fn validate_tree_membership(
    staging_root: &Path,
    operations: &[CopyOperation],
    errors: &mut Vec<PackageValidationError>,
) {
    let expected: BTreeSet<_> = operations
        .iter()
        .map(|operation| operation.target_relative_path.clone())
        .collect();
    match collect_files(staging_root) {
        Ok(observed) => {
            for unexpected in observed.difference(&expected) {
                errors.push(crate::package_validator_result::error(
                    "VALIDATION_UNEXPECTED_STAGED_FILE",
                    "Staging contains a file that is not in the package plan",
                    None,
                    Some(&staging_root.join(unexpected)),
                ));
            }
            for missing in expected.difference(&observed) {
                errors.push(crate::package_validator_result::error(
                    "VALIDATION_PLANNED_FILE_MISSING",
                    "A planned staged file is missing",
                    None,
                    Some(&staging_root.join(missing)),
                ));
            }
        }
        Err(error) => errors.push(error),
    }
}

fn collect_files(root: &Path) -> Result<BTreeSet<PathBuf>, PackageValidationError> {
    let mut files = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory).map_err(|error| {
            crate::package_validator_result::error(
                "VALIDATION_STAGING_READ_FAILED",
                format!("Cannot inspect staging tree: {error}"),
                None,
                Some(&directory),
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                crate::package_validator_result::error(
                    "VALIDATION_STAGING_READ_FAILED",
                    format!("Cannot inspect staging entry: {error}"),
                    None,
                    Some(&directory),
                )
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                crate::package_validator_result::error(
                    "VALIDATION_STAGING_METADATA_FAILED",
                    format!("Cannot inspect staging metadata: {error}"),
                    None,
                    Some(&path),
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(crate::package_validator_result::error(
                    "VALIDATION_STAGING_SYMLINK",
                    "Symlinks are forbidden inside a staged package",
                    None,
                    Some(&path),
                ));
            }
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|_| {
                    crate::package_validator_result::error(
                        "VALIDATION_STAGING_PATH_ESCAPE",
                        "Staging entry escaped its root",
                        None,
                        Some(&path),
                    )
                })?;
                if !safe_relative(relative) {
                    return Err(crate::package_validator_result::error(
                        "VALIDATION_STAGING_PATH_UNSAFE",
                        "Staging contains an unsafe relative path",
                        None,
                        Some(&path),
                    ));
                }
                files.insert(relative.to_path_buf());
            }
        }
    }
    Ok(files)
}

pub(crate) fn hash_regular_file(path: &Path) -> Result<(String, u64), FileFailure> {
    let metadata = fs::symlink_metadata(path).map_err(|error| FileFailure {
        code: "VALIDATION_FILE_UNAVAILABLE",
        message: format!("Cannot inspect staged file: {error}"),
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(FileFailure {
            code: "VALIDATION_FILE_NOT_REGULAR",
            message: "Staged path must be a regular non-symlink file".to_string(),
        });
    }
    let mut file = File::open(path).map_err(|error| FileFailure {
        code: "VALIDATION_FILE_READ_FAILED",
        message: format!("Cannot open staged file: {error}"),
    })?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| FileFailure {
            code: "VALIDATION_FILE_READ_FAILED",
            message: format!("Cannot read staged file: {error}"),
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

fn inspect_regular_file(
    path: &Path,
    verification_method: &str,
) -> Result<(Option<String>, u64), FileFailure> {
    match verification_method {
        rescue_packaging::VERIFY_SHA256_AND_SIZE => {
            hash_regular_file(path).map(|(hash, size)| (Some(hash), size))
        }
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE => {
            regular_file_size(path).map(|size| (None, size))
        }
        _ => Err(FileFailure {
            code: "VALIDATION_VERIFICATION_POLICY_UNSUPPORTED",
            message: "File validation policy is unsupported".to_string(),
        }),
    }
}

fn regular_file_size(path: &Path) -> Result<u64, FileFailure> {
    let metadata = fs::symlink_metadata(path).map_err(|error| FileFailure {
        code: "VALIDATION_FILE_UNAVAILABLE",
        message: format!("Cannot inspect staged file: {error}"),
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(FileFailure {
            code: "VALIDATION_FILE_NOT_REGULAR",
            message: "Staged path must be a regular non-symlink file".to_string(),
        });
    }
    Ok(metadata.len())
}

pub(crate) struct FileFailure {
    pub code: &'static str,
    pub message: String,
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
