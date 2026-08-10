use crate::{DirectoryValidationRecord, PackageValidationError};
use rescue_packaging::PackagePlan;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct DirectoryValidationOutcome {
    pub records: Vec<DirectoryValidationRecord>,
    pub errors: Vec<PackageValidationError>,
}

pub(crate) fn validate_directories(
    staging_root: &Path,
    plan: &PackagePlan,
) -> DirectoryValidationOutcome {
    let marker_count = plan
        .directory_operations
        .iter()
        .filter(|operation| {
            operation.target_relative_path == Path::new("Ableton Project Info")
                && operation.purpose == "ableton_project_marker"
        })
        .count();
    let mut errors = Vec::new();
    if marker_count != 1 {
        errors.push(crate::package_validator_result::error(
            "VALIDATION_ABLETON_PROJECT_MARKER_NOT_PLANNED",
            "Exactly one Ableton Project Info directory must be planned",
            None,
            Some(Path::new("Ableton Project Info")),
        ));
    }

    let records = plan
        .directory_operations
        .iter()
        .map(|operation| {
            let path = staging_root.join(&operation.target_relative_path);
            let status = match fs::symlink_metadata(&path) {
                Ok(metadata)
                    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() =>
                {
                    "verified"
                }
                Ok(_) => {
                    errors.push(crate::package_validator_result::error(
                        "VALIDATION_PLANNED_DIRECTORY_INVALID",
                        "Planned project directory is not a real non-symlink directory",
                        Some(&operation.operation_id),
                        Some(&path),
                    ));
                    "invalid"
                }
                Err(_) => {
                    let code = if operation.purpose == "ableton_project_marker" {
                        "VALIDATION_ABLETON_PROJECT_MARKER_MISSING"
                    } else {
                        "VALIDATION_PLANNED_DIRECTORY_MISSING"
                    };
                    errors.push(crate::package_validator_result::error(
                        code,
                        "Planned project directory is missing",
                        Some(&operation.operation_id),
                        Some(&path),
                    ));
                    "missing"
                }
            };
            DirectoryValidationRecord {
                operation_id: operation.operation_id.clone(),
                target_relative_path: PathBuf::from(&operation.target_relative_path),
                purpose: operation.purpose.clone(),
                directory_status: status.to_string(),
            }
        })
        .collect();
    DirectoryValidationOutcome { records, errors }
}
