use crate::{LaboratoryPackageError, LaboratoryPackageRequest};
use std::fs;
use std::path::{Component, Path};

pub(crate) fn validate_request(request: &LaboratoryPackageRequest) -> Vec<LaboratoryPackageError> {
    let mut errors = Vec::new();
    if request.run_id.trim().is_empty() {
        errors.push(error("PIPELINE_RUN_ID_EMPTY", "Run ID must not be empty"));
    }
    if request.scan_roots.is_empty() {
        errors.push(error(
            "PIPELINE_SCAN_SCOPE_EMPTY",
            "At least one explicitly selected scan root is required",
        ));
    }
    if request.max_scan_entries == 0 || request.max_scan_entries > 1_000_000 {
        errors.push(error(
            "PIPELINE_SCAN_LIMIT_INVALID",
            "Laboratory scan limit must be between 1 and 1,000,000 entries",
        ));
    }
    validate_source(&request.source_als_path, &mut errors);
    for root in &request.scan_roots {
        validate_scan_root(root, &mut errors);
    }
    validate_output_paths(request, &mut errors);
    errors
}

fn validate_source(path: &Path, errors: &mut Vec<LaboratoryPackageError>) {
    if !safe_absolute(path) {
        errors.push(error(
            "PIPELINE_SOURCE_PATH_UNSAFE",
            "Source ALS path must be absolute without traversal",
        ));
        return;
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() && !metadata.file_type().is_symlink() => {}
        _ => errors.push(error(
            "PIPELINE_SOURCE_INVALID",
            "Source ALS must be an existing regular non-symlink file",
        )),
    }
}

fn validate_scan_root(path: &Path, errors: &mut Vec<LaboratoryPackageError>) {
    if !safe_absolute(path) || path.parent().is_none() {
        errors.push(error(
            "PIPELINE_SCAN_ROOT_UNSAFE",
            "Laboratory scan roots must be bounded absolute directories",
        ));
        return;
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
        _ => errors.push(error(
            "PIPELINE_SCAN_ROOT_INVALID",
            "Every scan root must be an existing non-symlink directory",
        )),
    }
}

fn validate_output_paths(
    request: &LaboratoryPackageRequest,
    errors: &mut Vec<LaboratoryPackageError>,
) {
    for path in [
        &request.staging_root,
        &request.target_project_root,
        &request.private_ledger_path,
    ] {
        if !safe_absolute(path) {
            errors.push(error(
                "PIPELINE_OUTPUT_PATH_UNSAFE",
                "Every output path must be absolute without traversal",
            ));
        }
    }
    if request.staging_root == request.target_project_root
        || request.source_als_path.starts_with(&request.staging_root)
        || request
            .source_als_path
            .starts_with(&request.target_project_root)
        || request
            .private_ledger_path
            .starts_with(&request.staging_root)
        || request
            .private_ledger_path
            .starts_with(&request.target_project_root)
    {
        errors.push(error(
            "PIPELINE_OUTPUT_SCOPE_CONFLICT",
            "Source, staging, target, and private ledger scopes must be isolated",
        ));
    }
    if request.staging_root.exists() || request.target_project_root.exists() {
        errors.push(error(
            "PIPELINE_OUTPUT_ALREADY_EXISTS",
            "Fresh laboratory staging and target paths must not exist",
        ));
    }
    if request.private_ledger_path.exists() {
        errors.push(error(
            "PIPELINE_PRIVATE_LEDGER_EXISTS",
            "Fresh laboratory private ledger path must not exist",
        ));
    }
    for parent in [
        request.staging_root.parent(),
        request.target_project_root.parent(),
        request.private_ledger_path.parent(),
    ] {
        match parent.and_then(valid_directory) {
            Some(()) => {}
            None => errors.push(error(
                "PIPELINE_OUTPUT_PARENT_INVALID",
                "All output parent directories must already exist and be non-symlink directories",
            )),
        }
    }
}

fn valid_directory(path: &Path) -> Option<()> {
    let metadata = fs::symlink_metadata(path).ok()?;
    (metadata.file_type().is_dir() && !metadata.file_type().is_symlink()).then_some(())
}

fn safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn error(code: &str, message: &str) -> LaboratoryPackageError {
    LaboratoryPackageError {
        error_code: code.to_string(),
        stage: "request_validation".to_string(),
        message: message.to_string(),
    }
}
