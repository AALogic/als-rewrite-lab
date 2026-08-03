use crate::{DirectoryExecutionRecord, StagingExecutionError};
use rescue_packaging::CreateDirectoryOperation;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) struct DirectoryCreationFailure {
    pub records: Vec<DirectoryExecutionRecord>,
    pub error: StagingExecutionError,
}

pub(crate) fn create_directories(
    staging_root: &Path,
    operations: &[CreateDirectoryOperation],
) -> Result<Vec<DirectoryExecutionRecord>, Box<DirectoryCreationFailure>> {
    let mut records = Vec::new();
    for operation in operations {
        let path = staging_root.join(&operation.target_relative_path);
        if let Err(error) = fs::create_dir(&path) {
            records.push(record(operation, "failed"));
            return Err(Box::new(DirectoryCreationFailure {
                records,
                error: staging_error(
                    "STAGING_DIRECTORY_CREATE_FAILED",
                    format!("Cannot create planned project directory: {error}"),
                    Some(operation.operation_id.clone()),
                    Some(path),
                ),
            }));
        }
        records.push(record(operation, "created"));
    }
    Ok(records)
}

pub(crate) fn validate_directories(
    expected_count: usize,
    operations: &[CreateDirectoryOperation],
) -> Vec<StagingExecutionError> {
    let mut errors = Vec::new();
    if expected_count != operations.len() {
        errors.push(staging_error(
            "STAGING_DIRECTORY_COUNT_MISMATCH",
            "Plan metadata does not match its directory operations".to_string(),
            None,
            None,
        ));
    }
    let mut ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for operation in operations {
        if !ids.insert(operation.operation_id.as_str())
            || !targets.insert(operation.target_relative_path.clone())
        {
            errors.push(staging_error(
                "STAGING_DUPLICATE_DIRECTORY_OPERATION",
                "Directory operation IDs and targets must be unique".to_string(),
                Some(operation.operation_id.clone()),
                Some(operation.target_relative_path.clone()),
            ));
        }
        if !safe_relative(&operation.target_relative_path)
            || operation.collision_policy != "fail_if_exists"
        {
            errors.push(staging_error(
                "STAGING_DIRECTORY_OPERATION_UNSAFE",
                "Directory operation must use a safe path and fail-if-exists policy".to_string(),
                Some(operation.operation_id.clone()),
                Some(operation.target_relative_path.clone()),
            ));
        }
    }
    if marker_count(operations) != 1 {
        errors.push(staging_error(
            "STAGING_ABLETON_PROJECT_MARKER_NOT_PLANNED",
            "Exactly one Ableton Project Info directory must be planned".to_string(),
            None,
            Some(PathBuf::from("Ableton Project Info")),
        ));
    }
    errors
}

fn marker_count(operations: &[CreateDirectoryOperation]) -> usize {
    operations
        .iter()
        .filter(|operation| {
            operation.target_relative_path == Path::new("Ableton Project Info")
                && operation.purpose == "ableton_project_marker"
        })
        .count()
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn record(operation: &CreateDirectoryOperation, status: &str) -> DirectoryExecutionRecord {
    DirectoryExecutionRecord {
        operation_id: operation.operation_id.clone(),
        target_relative_path: operation.target_relative_path.clone(),
        purpose: operation.purpose.clone(),
        operation_status: status.to_string(),
    }
}

fn staging_error(
    code: &str,
    message: String,
    operation_id: Option<String>,
    path: Option<PathBuf>,
) -> StagingExecutionError {
    StagingExecutionError {
        error_code: code.to_string(),
        message,
        operation_id,
        path,
    }
}
