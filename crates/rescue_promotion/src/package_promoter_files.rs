use crate::{PackagePromotionError, PromotedFileRecord};
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_validation::PackageValidationResult;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

struct ExpectedFile {
    sha256: Option<String>,
    size: u64,
    verification_method: String,
}

pub(crate) struct RootVerification {
    pub records: Vec<PromotedFileRecord>,
    pub errors: Vec<PackagePromotionError>,
}

pub(crate) fn verify_package_root(
    root: &Path,
    plan: &PackagePlan,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
) -> RootVerification {
    let expected = match expected_files(plan, validation, manifests) {
        Ok(expected) => expected,
        Err(error) => {
            return RootVerification {
                records: Vec::new(),
                errors: vec![error],
            }
        }
    };
    let mut errors = Vec::new();
    match collect_files(root) {
        Ok(observed) => {
            let expected_paths: BTreeSet<_> = expected.keys().cloned().collect();
            for path in observed.difference(&expected_paths) {
                errors.push(error(
                    "PROMOTION_UNEXPECTED_FILE",
                    "Package contains an unexpected file",
                    Some(&root.join(path)),
                ));
            }
            for path in expected_paths.difference(&observed) {
                errors.push(error(
                    "PROMOTION_EXPECTED_FILE_MISSING",
                    "Package is missing an expected file",
                    Some(&root.join(path)),
                ));
            }
        }
        Err(error) => errors.push(error),
    }
    let mut records = Vec::new();
    for (relative, expected_file) in expected {
        let path = root.join(&relative);
        match crate::package_promoter_verification::inspect_regular_file(
            &path,
            &expected_file.verification_method,
        ) {
            Ok((hash, size)) if hash == expected_file.sha256 && size == expected_file.size => {
                records.push(record(
                    relative,
                    expected_file,
                    hash,
                    Some(size),
                    "verified",
                ));
            }
            Ok((hash, size)) => {
                errors.push(error(
                    "PROMOTION_FILE_MISMATCH",
                    "Package file changed after validation",
                    Some(&path),
                ));
                records.push(record(
                    relative,
                    expected_file,
                    hash,
                    Some(size),
                    "mismatch",
                ));
            }
            Err(file_error) => {
                errors.push(file_error);
                records.push(record(
                    relative,
                    expected_file,
                    None,
                    None,
                    "missing_or_invalid",
                ));
            }
        }
    }
    RootVerification { records, errors }
}

pub(crate) fn verify_private_ledger(
    manifests: &ManifestWriteResult,
) -> Result<(), PackagePromotionError> {
    let expected_hash = manifests.private_ledger_sha256.as_deref().ok_or_else(|| {
        error(
            "PROMOTION_PRIVATE_LEDGER_EVIDENCE_MISSING",
            "Private ledger hash is missing",
            Some(&manifests.private_ledger_path),
        )
    })?;
    let expected_size = manifests.private_ledger_size.ok_or_else(|| {
        error(
            "PROMOTION_PRIVATE_LEDGER_EVIDENCE_MISSING",
            "Private ledger size is missing",
            Some(&manifests.private_ledger_path),
        )
    })?;
    let (hash, size) =
        crate::package_promoter_verification::hash_regular_file(&manifests.private_ledger_path)?;
    if hash != expected_hash || size != expected_size {
        return Err(error(
            "PROMOTION_PRIVATE_LEDGER_MISMATCH",
            "Private ledger changed after it was written",
            Some(&manifests.private_ledger_path),
        ));
    }
    Ok(())
}

pub(crate) fn verify_original_source(plan: &PackagePlan) -> Result<(), PackagePromotionError> {
    let (hash, _) =
        crate::package_promoter_verification::hash_regular_file(&plan.source_als.source_als_path)?;
    if hash != plan.source_als.source_file_hash {
        return Err(error(
            "ORIGINAL_FILE_CHANGED",
            "Original ALS changed before package promotion",
            Some(&plan.source_als.source_als_path),
        ));
    }
    Ok(())
}

fn expected_files(
    plan: &PackagePlan,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
) -> Result<BTreeMap<PathBuf, ExpectedFile>, PackagePromotionError> {
    let mut expected = BTreeMap::new();
    for record in &validation.file_records {
        if record.file_status != "verified" {
            return Err(error(
                "PROMOTION_VALIDATION_RECORD_INVALID",
                "All validation file records must be verified",
                Some(&record.target_relative_path),
            ));
        }
        let hash_valid = match record.verification_method.as_str() {
            rescue_packaging::VERIFY_SHA256_AND_SIZE => record.observed_sha256.is_some(),
            rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE => record.observed_sha256.is_none(),
            _ => false,
        };
        if !hash_valid {
            return Err(error(
                "PROMOTION_VALIDATION_RECORD_INVALID",
                "Validation evidence does not match its verification method",
                Some(&record.target_relative_path),
            ));
        }
        let size = record.observed_size.ok_or_else(|| {
            error(
                "PROMOTION_VALIDATION_RECORD_INVALID",
                "Validation file size is missing",
                Some(&record.target_relative_path),
            )
        })?;
        expected.insert(
            record.target_relative_path.clone(),
            ExpectedFile {
                sha256: record.observed_sha256.clone(),
                size,
                verification_method: record.verification_method.clone(),
            },
        );
    }
    if expected.len() != plan.copy_operations.len() {
        return Err(error(
            "PROMOTION_VALIDATION_RECORD_INVALID",
            "Validated file set does not match the package plan",
            None,
        ));
    }
    let manifest_hash = manifests.package_manifest_sha256.clone().ok_or_else(|| {
        error(
            "PROMOTION_MANIFEST_EVIDENCE_INCOMPLETE",
            "Package manifest hash is missing",
            Some(&manifests.package_manifest_relative_path),
        )
    })?;
    let manifest_size = manifests.package_manifest_size.ok_or_else(|| {
        error(
            "PROMOTION_MANIFEST_EVIDENCE_INCOMPLETE",
            "Package manifest size is missing",
            Some(&manifests.package_manifest_relative_path),
        )
    })?;
    if expected
        .insert(
            manifests.package_manifest_relative_path.clone(),
            ExpectedFile {
                sha256: Some(manifest_hash),
                size: manifest_size,
                verification_method: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
            },
        )
        .is_some()
    {
        return Err(error(
            "PROMOTION_MANIFEST_TARGET_COLLISION",
            "Package manifest collides with a validated package file",
            Some(&manifests.package_manifest_relative_path),
        ));
    }
    Ok(expected)
}

fn collect_files(root: &Path) -> Result<BTreeSet<PathBuf>, PackagePromotionError> {
    let metadata = fs::symlink_metadata(root).map_err(|_| {
        error(
            "PROMOTION_ROOT_INVALID",
            "Cannot inspect package root",
            Some(root),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(error(
            "PROMOTION_ROOT_INVALID",
            "Package root must be a non-symlink directory",
            Some(root),
        ));
    }
    let mut files = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory).map_err(|read_error| {
            error(
                "PROMOTION_TREE_READ_FAILED",
                &format!("Cannot read package tree: {read_error}"),
                Some(&directory),
            )
        })?;
        for entry in entries {
            let path = entry
                .map_err(|_| {
                    error(
                        "PROMOTION_TREE_READ_FAILED",
                        "Cannot read package entry",
                        Some(&directory),
                    )
                })?
                .path();
            let metadata = fs::symlink_metadata(&path).map_err(|_| {
                error(
                    "PROMOTION_TREE_READ_FAILED",
                    "Cannot inspect package entry",
                    Some(&path),
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(error(
                    "PROMOTION_SYMLINK_FORBIDDEN",
                    "Symlinks are forbidden inside a package",
                    Some(&path),
                ));
            }
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|_| {
                    error(
                        "PROMOTION_PATH_ESCAPE",
                        "Package entry escaped its root",
                        Some(&path),
                    )
                })?;
                if !safe_relative(relative) {
                    return Err(error(
                        "PROMOTION_PATH_UNSAFE",
                        "Package entry has an unsafe relative path",
                        Some(&path),
                    ));
                }
                files.insert(relative.to_path_buf());
            }
        }
    }
    Ok(files)
}

fn record(
    relative_path: PathBuf,
    expected: ExpectedFile,
    observed_hash: Option<String>,
    observed_size: Option<u64>,
    status: &str,
) -> PromotedFileRecord {
    PromotedFileRecord {
        relative_path,
        expected_sha256: expected.sha256,
        observed_sha256: observed_hash,
        expected_size: expected.size,
        observed_size,
        verification_method: expected.verification_method,
        file_status: status.to_string(),
    }
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn error(code: &str, message: &str, path: Option<&Path>) -> PackagePromotionError {
    crate::package_promoter_result::error(code, message, path)
}
