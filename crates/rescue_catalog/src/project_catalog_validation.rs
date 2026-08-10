use crate::{
    project_catalog_identity, ProjectCatalogBuildRequest, ProjectCatalogError, ProjectScanResult,
    PROJECT_SCANNER_VERSION, PROJECT_SCAN_TRAVERSAL_POLICY,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn validate_request(
    request: &ProjectCatalogBuildRequest,
) -> Result<(), ProjectCatalogError> {
    if request.snapshot_id.trim().is_empty() {
        return Err(error(
            "CATALOG_EMPTY_SNAPSHOT_ID",
            "Catalog snapshot ID must not be empty.",
        ));
    }
    let scan = &request.scan_result;
    if scan.metadata.scanner_version != PROJECT_SCANNER_VERSION {
        return Err(error(
            "CATALOG_UNSUPPORTED_SCANNER_VERSION",
            "The ProjectScanResult scanner version is not supported.",
        ));
    }
    if scan.metadata.traversal_policy_version != PROJECT_SCAN_TRAVERSAL_POLICY {
        return Err(error(
            "CATALOG_UNSUPPORTED_TRAVERSAL_POLICY",
            "The ProjectScanResult traversal policy is not supported.",
        ));
    }
    match scan.metadata.scan_status.as_str() {
        "failed" => {
            return Err(error(
                "CATALOG_SOURCE_SCAN_FAILED",
                "A failed project scan cannot produce a trusted catalog.",
            ));
        }
        "complete" | "partial" | "cancelled" => {}
        _ => {
            return Err(error(
                "CATALOG_UNKNOWN_SOURCE_STATUS",
                "The ProjectScanResult status is unknown.",
            ));
        }
    }
    validate_counts(scan)?;
    validate_observations(scan)
}

fn validate_counts(scan: &ProjectScanResult) -> Result<(), ProjectCatalogError> {
    let metadata = &scan.metadata;
    if metadata.als_file_count != scan.als_files.len()
        || metadata.project_marker_count != scan.project_markers.len()
        || metadata.warning_count != scan.warnings.len()
        || metadata.error_count != scan.errors.len()
    {
        return Err(error(
            "CATALOG_SOURCE_COUNTS_INCONSISTENT",
            "ProjectScanResult metadata counts do not match its records.",
        ));
    }
    Ok(())
}

fn validate_observations(scan: &ProjectScanResult) -> Result<(), ProjectCatalogError> {
    let mut ids = HashSet::new();
    let mut native_paths = HashSet::new();
    for observation in &scan.als_files {
        validate_absolute(&observation.source_root)?;
        validate_absolute(&observation.native_path)?;
        validate_utf8_paths(&[&observation.source_root, &observation.native_path])?;
        if !ids.insert(observation.observation_id.as_str()) {
            return Err(duplicate_id());
        }
        if !native_paths.insert(observation.native_path.clone()) {
            return Err(duplicate_path());
        }
    }
    for marker in &scan.project_markers {
        validate_absolute(&marker.source_root)?;
        validate_absolute(&marker.project_root_candidate)?;
        validate_absolute(&marker.marker_path)?;
        validate_utf8_paths(&[
            &marker.source_root,
            &marker.project_root_candidate,
            &marker.marker_path,
        ])?;
        if !ids.insert(marker.marker_observation_id.as_str()) {
            return Err(duplicate_id());
        }
        if !native_paths.insert(marker.marker_path.clone()) {
            return Err(duplicate_path());
        }
        if marker.marker_name != "Ableton Project Info"
            || marker.marker_path.parent() != Some(marker.project_root_candidate.as_path())
        {
            return Err(error(
                "CATALOG_MARKER_ROOT_INVALID",
                "A marker is not an exact Ableton Project Info child of its project root.",
            ));
        }
    }
    Ok(())
}

fn validate_absolute(path: &Path) -> Result<(), ProjectCatalogError> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(error(
            "CATALOG_PATH_NOT_ABSOLUTE",
            "Catalog native paths and source roots must be absolute.",
        ))
    }
}

fn validate_utf8_paths(paths: &[&PathBuf]) -> Result<(), ProjectCatalogError> {
    for path in paths {
        project_catalog_identity::exact_path(path)?;
    }
    Ok(())
}

fn duplicate_id() -> ProjectCatalogError {
    error(
        "CATALOG_DUPLICATE_OBSERVATION_ID",
        "ProjectScanResult contains a duplicate observation ID.",
    )
}

fn duplicate_path() -> ProjectCatalogError {
    error(
        "CATALOG_DUPLICATE_NATIVE_PATH",
        "ProjectScanResult contains a duplicate native observation path.",
    )
}

fn error(error_code: &str, message: &str) -> ProjectCatalogError {
    ProjectCatalogError {
        error_code: error_code.to_string(),
        message: message.to_string(),
    }
}
