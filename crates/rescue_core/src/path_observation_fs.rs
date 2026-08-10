use crate::{CandidatePathObservation, DependencyRef, PathObservationWarning};
use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn observe_native_path(
    candidate_id: String,
    basis: &str,
    path: &Path,
    dependency: &DependencyRef,
    next_warning_id: &mut usize,
) -> CandidatePathObservation {
    let expected = expected_size(dependency);
    match fs::symlink_metadata(path) {
        Ok(metadata) => observed_entry(
            candidate_id,
            basis,
            path,
            metadata,
            expected,
            dependency,
            next_warning_id,
        ),
        Err(error) => failed_entry(
            candidate_id,
            basis,
            path,
            error,
            expected,
            dependency,
            next_warning_id,
        ),
    }
}

fn observed_entry(
    candidate_id: String,
    basis: &str,
    path: &Path,
    metadata: fs::Metadata,
    expected: Option<u64>,
    dependency: &DependencyRef,
    next_warning_id: &mut usize,
) -> CandidatePathObservation {
    let file_type = metadata.file_type();
    let (availability, entry_kind, observed) = if file_type.is_symlink() {
        ("existing_symlink", "symlink", None)
    } else if file_type.is_file() {
        (
            "existing_regular_file",
            "regular_file",
            Some(metadata.len()),
        )
    } else if file_type.is_dir() {
        ("existing_directory", "directory", None)
    } else {
        ("not_checked", "other", None)
    };
    let size_status = size_status(expected, observed, entry_kind);
    let mut warnings = Vec::new();
    if entry_kind == "symlink" {
        warnings.push(new_warning(
            next_warning_id,
            "PATH_CANDIDATE_IS_SYMLINK",
            "Path candidate is a symlink and its target was not followed",
            dependency,
            Some(candidate_id.clone()),
        ));
    }
    if size_status == "differs_from_expected_size" {
        warnings.push(new_warning(
            next_warning_id,
            "PATH_CANDIDATE_SIZE_DIFFERS",
            "Observed file size differs from ALS size evidence",
            dependency,
            Some(candidate_id.clone()),
        ));
    }
    CandidatePathObservation {
        candidate_id,
        candidate_basis: basis.to_string(),
        candidate_path: path.to_string_lossy().to_string(),
        platform_status: "checkable_on_current_platform".to_string(),
        safety_status: "safe_for_metadata_read".to_string(),
        availability_status: availability.to_string(),
        entry_kind: entry_kind.to_string(),
        size_evidence_status: size_status.to_string(),
        observed_file_size: observed,
        expected_file_size: expected,
        evidence_notes: vec!["symlink_metadata_only".to_string()],
        warnings,
    }
}

fn failed_entry(
    candidate_id: String,
    basis: &str,
    path: &Path,
    error: io::Error,
    expected: Option<u64>,
    dependency: &DependencyRef,
    next_warning_id: &mut usize,
) -> CandidatePathObservation {
    let (availability, code, message) = if error.kind() == io::ErrorKind::NotFound {
        (
            "missing",
            "PATH_CANDIDATE_NOT_FOUND",
            "Path candidate was not found",
        )
    } else {
        (
            "inaccessible",
            "PATH_CANDIDATE_INACCESSIBLE",
            "Path candidate metadata could not be read",
        )
    };
    let warning = new_warning(
        next_warning_id,
        code,
        message,
        dependency,
        Some(candidate_id.clone()),
    );
    CandidatePathObservation {
        candidate_id,
        candidate_basis: basis.to_string(),
        candidate_path: path.to_string_lossy().to_string(),
        platform_status: "checkable_on_current_platform".to_string(),
        safety_status: "safe_for_metadata_read".to_string(),
        availability_status: availability.to_string(),
        entry_kind: "unknown".to_string(),
        size_evidence_status: if expected.is_some() {
            "observed_size_unavailable"
        } else {
            "expected_size_unavailable"
        }
        .to_string(),
        observed_file_size: None,
        expected_file_size: expected,
        evidence_notes: vec![format!("io_error_kind:{:?}", error.kind())],
        warnings: vec![warning],
    }
}

pub(crate) fn new_warning(
    next_warning_id: &mut usize,
    code: &str,
    message: &str,
    dependency: &DependencyRef,
    candidate_id: Option<String>,
) -> PathObservationWarning {
    let warning = PathObservationWarning {
        warning_id: *next_warning_id,
        warning_code: code.to_string(),
        severity: "warning".to_string(),
        message: message.to_string(),
        dependency_id: Some(dependency.dependency_id.clone()),
        als_ref_id: Some(dependency.als_ref_id),
        candidate_id,
        evidence_status: "observed".to_string(),
    };
    *next_warning_id += 1;
    warning
}

fn expected_size(dependency: &DependencyRef) -> Option<u64> {
    dependency
        .original_file_size
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
}

fn size_status(expected: Option<u64>, observed: Option<u64>, entry_kind: &str) -> &'static str {
    if entry_kind != "regular_file" {
        return "not_applicable";
    }
    match (expected, observed) {
        (Some(left), Some(right)) if left == right => "matches_expected_size",
        (Some(_), Some(_)) => "differs_from_expected_size",
        (None, Some(_)) => "expected_size_unavailable",
        (Some(_), None) => "observed_size_unavailable",
        (None, None) => "not_applicable",
    }
}
