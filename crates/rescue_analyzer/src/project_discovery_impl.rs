use crate::{
    ProjectDiscoveryError, ProjectDiscoveryMetadata, ProjectDiscoveryRequest,
    ProjectDiscoveryResult, ProjectDiscoveryWarning, ProjectRootCandidate,
    PROJECT_DISCOVERY_VERSION,
};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

const PROJECT_MARKER: &str = "Ableton Project Info";

pub(crate) fn discover_project_impl(request: &ProjectDiscoveryRequest) -> ProjectDiscoveryResult {
    if let Some(error) = source_error(&request.source_als_path) {
        return fatal_result(request, error);
    }
    let mut candidates = Vec::new();
    let mut warnings = Vec::new();
    let mut ancestors_checked = 0;
    let mut current = request.source_als_path.parent();
    while let Some(ancestor) = current {
        inspect_marker(ancestor, ancestors_checked, &mut candidates, &mut warnings);
        ancestors_checked += 1;
        current = ancestor.parent();
    }
    completed_result(request, candidates, warnings, ancestors_checked)
}

fn source_error(path: &Path) -> Option<ProjectDiscoveryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Some(error(
            "PROJECT_DISCOVERY_SOURCE_IS_SYMLINK",
            "Selected ALS is a symlink and was not followed",
            path,
        )),
        Ok(metadata) if !metadata.is_file() => Some(error(
            "PROJECT_DISCOVERY_SOURCE_NOT_FILE",
            "Selected ALS path is not a regular file",
            path,
        )),
        Ok(_) => None,
        Err(io_error) if io_error.kind() == io::ErrorKind::NotFound => Some(error(
            "PROJECT_DISCOVERY_SOURCE_NOT_FOUND",
            "Selected ALS path was not found",
            path,
        )),
        Err(_) => Some(error(
            "PROJECT_DISCOVERY_SOURCE_METADATA_FAILED",
            "Selected ALS metadata could not be read",
            path,
        )),
    }
}

fn inspect_marker(
    ancestor: &Path,
    depth: usize,
    candidates: &mut Vec<ProjectRootCandidate>,
    warnings: &mut Vec<ProjectDiscoveryWarning>,
) {
    let marker = ancestor.join(PROJECT_MARKER);
    match fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            candidates.push(ProjectRootCandidate {
                candidate_path: ancestor.to_path_buf(),
                marker_path: marker,
                marker_status: "exact_directory_marker".to_string(),
                depth_from_set: depth,
            });
        }
        Ok(metadata) if metadata.file_type().is_symlink() => warnings.push(warning(
            warnings.len(),
            "PROJECT_MARKER_IS_SYMLINK",
            "Ableton Project Info marker is a symlink and was not followed",
            marker,
        )),
        Ok(_) => warnings.push(warning(
            warnings.len(),
            "PROJECT_MARKER_NOT_DIRECTORY",
            "Ableton Project Info marker exists but is not a directory",
            marker,
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => warnings.push(warning(
            warnings.len(),
            "PROJECT_MARKER_INACCESSIBLE",
            "Ableton Project Info marker metadata could not be read",
            marker,
        )),
    }
}

fn completed_result(
    request: &ProjectDiscoveryRequest,
    candidates: Vec<ProjectRootCandidate>,
    warnings: Vec<ProjectDiscoveryWarning>,
    ancestors_checked: usize,
) -> ProjectDiscoveryResult {
    let (confirmed, status) = match candidates.as_slice() {
        [candidate] => (Some(candidate.candidate_path.clone()), "confirmed"),
        [] => (None, "unknown"),
        _ => (None, "ambiguous"),
    };
    let set_location = confirmed
        .as_deref()
        .map(|root| set_location(&request.source_als_path, root))
        .unwrap_or("standalone_or_unidentified");
    ProjectDiscoveryResult {
        metadata: metadata(
            request,
            ancestors_checked,
            candidates.len(),
            warnings.len(),
            0,
        ),
        candidates,
        confirmed_project_root: confirmed,
        discovery_status: status.to_string(),
        set_location: set_location.to_string(),
        warnings,
        errors: Vec::new(),
    }
}

fn set_location(source: &Path, root: &Path) -> &'static str {
    let Some(parent) = source.parent() else {
        return "unknown";
    };
    let Ok(relative_parent) = parent.strip_prefix(root) else {
        return "unknown";
    };
    if relative_parent
        .components()
        .any(|component| matches!(component, Component::Normal(value) if value == "Backup"))
    {
        "backup_candidate"
    } else if relative_parent.as_os_str().is_empty() {
        "project_root"
    } else {
        "project_subdirectory"
    }
}

fn fatal_result(
    request: &ProjectDiscoveryRequest,
    error: ProjectDiscoveryError,
) -> ProjectDiscoveryResult {
    ProjectDiscoveryResult {
        metadata: metadata(request, 0, 0, 0, 1),
        candidates: Vec::new(),
        confirmed_project_root: None,
        discovery_status: "unsupported".to_string(),
        set_location: "unknown".to_string(),
        warnings: Vec::new(),
        errors: vec![error],
    }
}

fn metadata(
    request: &ProjectDiscoveryRequest,
    ancestors_checked: usize,
    candidate_count: usize,
    warning_count: usize,
    error_count: usize,
) -> ProjectDiscoveryMetadata {
    ProjectDiscoveryMetadata {
        discovery_version: PROJECT_DISCOVERY_VERSION.to_string(),
        source_als_path: request.source_als_path.clone(),
        ancestors_checked,
        candidate_count,
        warning_count,
        error_count,
    }
}

fn warning(warning_id: usize, code: &str, message: &str, path: PathBuf) -> ProjectDiscoveryWarning {
    ProjectDiscoveryWarning {
        warning_id,
        warning_code: code.to_string(),
        message: message.to_string(),
        path: Some(path),
        evidence_status: "observed".to_string(),
    }
}

fn error(code: &str, message: &str, path: &Path) -> ProjectDiscoveryError {
    ProjectDiscoveryError {
        error_code: code.to_string(),
        message: message.to_string(),
        path: Some(path.to_path_buf()),
    }
}
