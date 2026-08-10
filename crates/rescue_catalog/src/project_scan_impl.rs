use crate::project_scan_walk::{
    emit_progress, walk_project_roots, RawAlsObservation, RawMarkerObservation,
};
use crate::{
    ALSFileObservation, ProjectMarkerObservation, ProjectScanError, ProjectScanMetadata,
    ProjectScanObserver, ProjectScanRequest, ProjectScanResult, PROJECT_SCANNER_VERSION,
    PROJECT_SCAN_TRAVERSAL_POLICY,
};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn scan_projects_impl(
    request: &ProjectScanRequest,
    observer: &mut dyn ProjectScanObserver,
) -> ProjectScanResult {
    let request_errors = validate_request(request);
    if !request_errors.is_empty() {
        return failed_result(request, request_errors, observer);
    }
    let requested_roots = normalized_roots(&request.roots);
    let (accepted_roots, mut errors) = validate_roots(&requested_roots);
    if accepted_roots.is_empty() {
        errors.push(error(
            "PROJECT_SCAN_NO_ACCEPTED_ROOTS",
            "No requested root could be accepted",
            None,
        ));
        return failed_result(request, errors, observer);
    }
    let exclusions = normalized_paths(&request.excluded_roots);
    let mut walk = walk_project_roots(request, &accepted_roots, &exclusions, observer);
    let als_files = build_als_records(std::mem::take(&mut walk.als_files));
    let project_markers = build_marker_records(std::mem::take(&mut walk.project_markers));
    let scan_status = status(&walk, !errors.is_empty());
    let metadata = ProjectScanMetadata {
        scanner_version: PROJECT_SCANNER_VERSION.to_string(),
        traversal_policy_version: request.traversal_policy_version.clone(),
        scan_run_id: request.scan_run_id.clone(),
        scan_status: scan_status.to_string(),
        requested_root_count: request.roots.len(),
        accepted_root_count: accepted_roots.len(),
        directories_visited: walk.directories_visited,
        entries_visited: walk.entries_visited,
        als_file_count: als_files.len(),
        project_marker_count: project_markers.len(),
        skipped_symlink_count: walk.skipped_symlink_count,
        skipped_excluded_count: walk.skipped_excluded_count,
        warning_count: walk.warnings.len(),
        error_count: errors.len(),
    };
    emit_progress(request, scan_status, &walk, observer);
    ProjectScanResult {
        metadata,
        als_files,
        project_markers,
        warnings: walk.warnings,
        errors,
    }
}

fn validate_request(request: &ProjectScanRequest) -> Vec<ProjectScanError> {
    let mut errors = Vec::new();
    if request.scan_run_id.trim().is_empty() {
        errors.push(error(
            "PROJECT_SCAN_EMPTY_ID",
            "Scan run ID must not be empty",
            None,
        ));
    }
    if request.roots.is_empty() {
        errors.push(error(
            "PROJECT_SCAN_EMPTY_SCOPE",
            "At least one approved scan root is required",
            None,
        ));
    }
    if request.max_entries == 0 {
        errors.push(error(
            "PROJECT_SCAN_INVALID_ENTRY_LIMIT",
            "Entry limit must be positive",
            None,
        ));
    }
    if request.traversal_policy_version != PROJECT_SCAN_TRAVERSAL_POLICY {
        errors.push(error(
            "PROJECT_SCAN_UNSUPPORTED_POLICY",
            "Traversal policy version is not supported",
            None,
        ));
    }
    for exclusion in &request.excluded_roots {
        if !exclusion.is_absolute() {
            errors.push(error(
                "PROJECT_SCAN_EXCLUSION_NOT_ABSOLUTE",
                "Excluded root must be absolute",
                Some(exclusion),
            ));
        }
    }
    errors
}

fn validate_roots(roots: &[PathBuf]) -> (Vec<PathBuf>, Vec<ProjectScanError>) {
    let mut accepted = Vec::new();
    let mut errors = Vec::new();
    for root in roots {
        if !root.is_absolute() {
            errors.push(error(
                "PROJECT_SCAN_ROOT_NOT_ABSOLUTE",
                "Approved scan root must be absolute",
                Some(root),
            ));
            continue;
        }
        match validate_root(root) {
            Ok(()) => accepted.push(root.clone()),
            Err(root_error) => errors.push(root_error),
        }
    }
    (accepted, errors)
}

fn validate_root(root: &Path) -> Result<(), ProjectScanError> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if root_is_link(&metadata) => Err(error(
            "PROJECT_SCAN_ROOT_IS_SYMLINK",
            "Approved scan root cannot be a symlink or reparse point",
            Some(root),
        )),
        Ok(metadata) if !metadata.is_dir() => Err(error(
            "PROJECT_SCAN_ROOT_NOT_DIRECTORY",
            "Approved scan root is not a directory",
            Some(root),
        )),
        Ok(_) => Ok(()),
        Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => Err(error(
            "PROJECT_SCAN_ROOT_NOT_FOUND",
            "Approved scan root was not found",
            Some(root),
        )),
        Err(_) => Err(error(
            "PROJECT_SCAN_ROOT_METADATA_FAILED",
            "Approved scan root metadata could not be read",
            Some(root),
        )),
    }
}

fn root_is_link(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    false
}

fn normalized_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let sorted = normalized_paths(roots);
    let mut reduced: Vec<PathBuf> = Vec::new();
    for root in sorted {
        if !reduced.iter().any(|parent| root.starts_with(parent)) {
            reduced.push(root);
        }
    }
    reduced
}

fn normalized_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = paths.to_vec();
    paths.sort();
    paths.dedup();
    paths
}

fn build_als_records(mut raw: Vec<RawAlsObservation>) -> Vec<ALSFileObservation> {
    raw.sort_by(|left, right| left.native_path.cmp(&right.native_path));
    raw.into_iter()
        .enumerate()
        .map(|(index, item)| ALSFileObservation {
            observation_id: format!("als_observation_{index:06}"),
            source_root: item.source_root,
            native_path: item.native_path,
            relative_path: item.relative_path,
            filename: item.filename,
            extension: "als".to_string(),
            file_size: item.file_size,
            modified_time_unix_ms: item.modified_time_unix_ms,
            entry_kind: "regular_file".to_string(),
            observation_status: "metadata_observed".to_string(),
        })
        .collect()
}

fn build_marker_records(mut raw: Vec<RawMarkerObservation>) -> Vec<ProjectMarkerObservation> {
    raw.sort_by(|left, right| left.marker_path.cmp(&right.marker_path));
    raw.into_iter()
        .enumerate()
        .map(|(index, item)| ProjectMarkerObservation {
            marker_observation_id: format!("project_marker_{index:06}"),
            source_root: item.source_root,
            project_root_candidate: item.project_root_candidate,
            marker_path: item.marker_path,
            relative_marker_path: item.relative_marker_path,
            marker_name: "Ableton Project Info".to_string(),
            entry_kind: "directory".to_string(),
            observation_status: "exact_marker_observed".to_string(),
        })
        .collect()
}

fn status(walk: &crate::project_scan_walk::ProjectWalkResult, root_errors: bool) -> &'static str {
    if walk.cancelled {
        "cancelled"
    } else if walk.partial || root_errors {
        "partial"
    } else {
        "complete"
    }
}

fn failed_result(
    request: &ProjectScanRequest,
    errors: Vec<ProjectScanError>,
    observer: &mut dyn ProjectScanObserver,
) -> ProjectScanResult {
    let result = ProjectScanResult {
        metadata: ProjectScanMetadata {
            scanner_version: PROJECT_SCANNER_VERSION.to_string(),
            traversal_policy_version: request.traversal_policy_version.clone(),
            scan_run_id: request.scan_run_id.clone(),
            scan_status: "failed".to_string(),
            requested_root_count: request.roots.len(),
            accepted_root_count: 0,
            directories_visited: 0,
            entries_visited: 0,
            als_file_count: 0,
            project_marker_count: 0,
            skipped_symlink_count: 0,
            skipped_excluded_count: 0,
            warning_count: 0,
            error_count: errors.len(),
        },
        als_files: Vec::new(),
        project_markers: Vec::new(),
        warnings: Vec::new(),
        errors,
    };
    observer.on_progress(&crate::ProjectScanProgress {
        scan_run_id: request.scan_run_id.clone(),
        stage: "failed".to_string(),
        requested_root_count: request.roots.len(),
        roots_completed: 0,
        directories_visited: 0,
        entries_visited: 0,
        als_file_count: 0,
        project_marker_count: 0,
        warning_count: 0,
    });
    result
}

fn error(code: &str, message: &str, path: Option<&Path>) -> ProjectScanError {
    ProjectScanError {
        error_code: code.to_string(),
        message: message.to_string(),
        path: path.map(Path::to_path_buf),
    }
}
