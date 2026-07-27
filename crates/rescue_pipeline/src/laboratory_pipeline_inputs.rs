use crate::{LaboratoryPackageError, LaboratoryPackageRequest};
use rescue_analyzer::ProjectDiscoveryResult;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

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
    if path_is_occupied(&request.staging_root) || path_is_occupied(&request.target_project_root) {
        errors.push(error(
            "PIPELINE_OUTPUT_ALREADY_EXISTS",
            "Fresh laboratory staging and target paths must not exist",
        ));
    }
    if path_is_occupied(&request.private_ledger_path) {
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

pub(crate) fn validate_discovered_project(
    request: &LaboratoryPackageRequest,
    discovery: &ProjectDiscoveryResult,
) -> Option<LaboratoryPackageError> {
    let Some(project_root) = discovery.confirmed_project_root.as_deref() else {
        return Some(project_error(
            "PIPELINE_PROJECT_ROOT_UNCONFIRMED",
            "A confirmed Ableton Project root is required before any write-capable flow",
        ));
    };
    if discovery.discovery_status != "confirmed" {
        return Some(project_error(
            "PIPELINE_PROJECT_ROOT_UNCONFIRMED",
            "Project discovery did not produce one confirmed Ableton Project root",
        ));
    }
    let resolved_project_root = match fs::canonicalize(project_root) {
        Ok(path) => path,
        Err(_) => {
            return Some(project_error(
                "PIPELINE_OUTPUT_SCOPE_UNVERIFIED",
                "The confirmed Project root could not be resolved for output isolation",
            ))
        }
    };
    for path in [
        &request.staging_root,
        &request.target_project_root,
        &request.private_ledger_path,
    ] {
        match resolved_parent(path)
            .and_then(|parent| resolved_path_starts_with(&parent, &resolved_project_root))
        {
            Ok(true) => {
                return Some(project_error(
                    "PIPELINE_OUTPUT_INSIDE_SOURCE_PROJECT",
                    "Staging, target, and private ledger paths must remain outside the source Project root",
                ))
            }
            Ok(false) => {}
            Err(_) => {
                return Some(project_error(
                    "PIPELINE_OUTPUT_SCOPE_UNVERIFIED",
                    "An output parent could not be resolved for Project isolation",
                ))
            }
        }
    }
    None
}

fn resolved_parent(path: &Path) -> io::Result<PathBuf> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "output path does not have a parent",
        )
    })?;
    fs::canonicalize(parent)
}

fn resolved_path_starts_with(path: &Path, root: &Path) -> io::Result<bool> {
    let mut path_components = path.components();
    for root_component in root.components() {
        let Some(path_component) = path_components.next() else {
            return Ok(false);
        };
        if !components_equal(path_component, root_component)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn components_equal(left: Component<'_>, right: Component<'_>) -> io::Result<bool> {
    let left = left.as_os_str().to_str().ok_or_else(non_unicode_path)?;
    let right = right.as_os_str().to_str().ok_or_else(non_unicode_path)?;
    Ok(left.to_lowercase() == right.to_lowercase())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn components_equal(left: Component<'_>, right: Component<'_>) -> io::Result<bool> {
    Ok(left == right)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn non_unicode_path() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "resolved path is not valid Unicode",
    )
}

fn path_is_occupied(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error) => error.kind() != io::ErrorKind::NotFound,
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

fn project_error(code: &str, message: &str) -> LaboratoryPackageError {
    LaboratoryPackageError {
        error_code: code.to_string(),
        stage: "project_discovery".to_string(),
        message: message.to_string(),
    }
}
