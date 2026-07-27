use crate::path_observation_fs::{new_warning, observe_native_path};
use crate::path_observation_path::{
    has_parent_component, is_foreign_absolute, is_native_absolute, safe_relative_path,
};
use crate::{
    parse_als_path, CandidatePathObservation, DependencyRef, PathObservationContext,
    PathObservationWarning,
};
use std::path::{Path, PathBuf};

pub(crate) struct CandidateBundle {
    pub candidates: Vec<CandidatePathObservation>,
    pub warnings: Vec<PathObservationWarning>,
}

pub(crate) fn observe_candidates(
    dependency: &DependencyRef,
    context: &PathObservationContext,
    next_warning_id: &mut usize,
) -> CandidateBundle {
    let mut bundle = CandidateBundle {
        candidates: Vec::new(),
        warnings: Vec::new(),
    };
    add_project_candidate(dependency, context, next_warning_id, &mut bundle);
    add_direct_candidate(dependency, context, next_warning_id, &mut bundle);
    for candidate in &bundle.candidates {
        bundle.warnings.extend(candidate.warnings.iter().cloned());
    }
    bundle
}

fn add_project_candidate(
    dependency: &DependencyRef,
    context: &PathObservationContext,
    next_warning_id: &mut usize,
    bundle: &mut CandidateBundle,
) {
    let source = match dependency.relative_path_type.as_deref() {
        Some("0") => dependency
            .raw_path
            .as_deref()
            .map(|value| ("confirmed_project_root_plus_raw_path", value)),
        Some("3") => dependency
            .raw_relative_path
            .as_deref()
            .map(|value| ("confirmed_project_root_plus_raw_relative_path", value)),
        _ => None,
    };
    let Some((basis, raw)) = source.filter(|(_, value)| !value.is_empty()) else {
        return;
    };
    let Some(root) = context.confirmed_project_root.as_deref() else {
        bundle.warnings.push(new_warning(
            next_warning_id,
            "PATH_PROJECT_ROOT_UNAVAILABLE",
            "Project-relative path exists but no confirmed Project root was supplied",
            dependency,
            None,
        ));
        return;
    };
    let candidate_id = next_candidate_id(dependency, bundle.candidates.len());
    let candidate = project_candidate(
        candidate_id,
        basis,
        raw,
        root,
        dependency,
        context,
        next_warning_id,
    );
    bundle.candidates.push(candidate);
}

fn project_candidate(
    candidate_id: String,
    basis: &str,
    raw: &str,
    root: &Path,
    dependency: &DependencyRef,
    context: &PathObservationContext,
    next_warning_id: &mut usize,
) -> CandidatePathObservation {
    match safe_relative_path(raw, &context.host_platform) {
        Ok(relative) => observe_native_path(
            candidate_id,
            basis,
            &root.join(relative),
            dependency,
            next_warning_id,
        ),
        Err(reason) => rejected_candidate(
            candidate_id,
            basis,
            raw,
            reason,
            dependency,
            next_warning_id,
        ),
    }
}

fn add_direct_candidate(
    dependency: &DependencyRef,
    context: &PathObservationContext,
    next_warning_id: &mut usize,
    bundle: &mut CandidateBundle,
) {
    let Some(raw) = dependency
        .raw_path
        .as_deref()
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let parsed = parse_als_path(raw);
    let candidate_id = next_candidate_id(dependency, bundle.candidates.len());
    if is_native_absolute(&parsed.kind, &context.host_platform) {
        let path = PathBuf::from(raw);
        let candidate = if has_parent_component(&path) {
            rejected_candidate(
                candidate_id,
                "recorded_raw_absolute_path",
                raw,
                "parent_escape",
                dependency,
                next_warning_id,
            )
        } else {
            observe_native_path(
                candidate_id,
                "recorded_raw_absolute_path",
                &path,
                dependency,
                next_warning_id,
            )
        };
        bundle.candidates.push(candidate);
    } else if is_foreign_absolute(&parsed.kind, &context.host_platform) {
        bundle.candidates.push(foreign_candidate(
            candidate_id,
            raw,
            dependency,
            next_warning_id,
        ));
    }
}

fn rejected_candidate(
    candidate_id: String,
    basis: &str,
    raw: &str,
    reason: &'static str,
    dependency: &DependencyRef,
    next_warning_id: &mut usize,
) -> CandidatePathObservation {
    let (safety, code) = if reason == "parent_escape" {
        ("rejected_parent_escape", "PATH_CANDIDATE_PARENT_ESCAPE")
    } else {
        ("rejected_invalid_path", "PATH_CANDIDATE_INVALID")
    };
    let warning = new_warning(
        next_warning_id,
        code,
        "Path candidate failed lexical safety checks",
        dependency,
        Some(candidate_id.clone()),
    );
    CandidatePathObservation {
        candidate_id,
        candidate_basis: basis.to_string(),
        candidate_path: raw.to_string(),
        platform_status: "checkable_on_current_platform".to_string(),
        safety_status: safety.to_string(),
        availability_status: "not_checked".to_string(),
        entry_kind: "unknown".to_string(),
        size_evidence_status: "not_applicable".to_string(),
        observed_file_size: None,
        expected_file_size: None,
        evidence_notes: vec![reason.to_string()],
        warnings: vec![warning],
    }
}

fn foreign_candidate(
    candidate_id: String,
    raw: &str,
    dependency: &DependencyRef,
    next_warning_id: &mut usize,
) -> CandidatePathObservation {
    let warning = new_warning(
        next_warning_id,
        "PATH_CANDIDATE_FOREIGN_PLATFORM",
        "Absolute path belongs to a different platform and was not checked",
        dependency,
        Some(candidate_id.clone()),
    );
    CandidatePathObservation {
        candidate_id,
        candidate_basis: "recorded_raw_absolute_path".to_string(),
        candidate_path: raw.to_string(),
        platform_status: "foreign_platform_path".to_string(),
        safety_status: "not_checked".to_string(),
        availability_status: "not_checked".to_string(),
        entry_kind: "unknown".to_string(),
        size_evidence_status: "not_applicable".to_string(),
        observed_file_size: None,
        expected_file_size: None,
        evidence_notes: vec!["raw_path_preserved".to_string()],
        warnings: vec![warning],
    }
}

fn next_candidate_id(dependency: &DependencyRef, index: usize) -> String {
    format!("{}:candidate:{index}", dependency.dependency_id)
}
