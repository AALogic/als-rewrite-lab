use crate::path_observation_candidate::observe_candidates;
use crate::{
    parse_als_path, DependencyExtractionResult, DependencyPathObservation, PathObservationContext,
    PathObservationError, PathObservationMetadata, PathObservationResult, PathObservationWarning,
    PATH_OBSERVATION_MODEL_VERSION, PATH_OBSERVER_VERSION,
};

pub(crate) fn observe_dependency_paths_impl(
    extraction: &DependencyExtractionResult,
    context: &PathObservationContext,
) -> PathObservationResult {
    if extraction.extraction_metadata.dependency_ref_version != "0.1" {
        return fatal_result(
            extraction,
            context,
            "PATH_OBSERVATION_UNSUPPORTED_INPUT_MODEL",
            "PathObservation requires DependencyRef v0.1",
        );
    }
    if !extraction.errors.is_empty() {
        return fatal_result(
            extraction,
            context,
            "PATH_OBSERVATION_UNTRUSTED_INPUT",
            "Dependency extraction contains fatal errors",
        );
    }
    if let Err(message) = validate_context(context) {
        return fatal_result(
            extraction,
            context,
            "PATH_OBSERVATION_INVALID_CONTEXT",
            message,
        );
    }

    let mut warnings = Vec::new();
    let mut next_warning_id = 0;
    let dependency_observations = extraction
        .dependencies
        .iter()
        .map(|dependency| {
            let bundle = observe_candidates(dependency, context, &mut next_warning_id);
            warnings.extend(bundle.warnings.iter().cloned());
            DependencyPathObservation {
                dependency_id: dependency.dependency_id.clone(),
                als_ref_id: dependency.als_ref_id,
                raw_path: dependency.raw_path.clone(),
                raw_relative_path: dependency.raw_relative_path.clone(),
                parsed_raw_path: dependency.raw_path.clone().map(parse_als_path),
                parsed_raw_relative_path: dependency.raw_relative_path.clone().map(parse_als_path),
                availability_summary: availability_summary(&bundle.candidates),
                identity_status: "not_evaluated".to_string(),
                candidates: bundle.candidates,
                warnings: bundle.warnings,
            }
        })
        .collect();
    complete_result(extraction, context, dependency_observations, warnings)
}

fn validate_context(context: &PathObservationContext) -> Result<(), &'static str> {
    if context.host_platform != current_host_platform() {
        return Err("host_platform does not match the current runtime");
    }
    match (
        context.confirmed_project_root.as_deref(),
        context.project_root_basis.as_deref(),
    ) {
        (None, None) => Ok(()),
        (Some(root), Some(basis)) if root.is_absolute() && allowed_basis(basis) => Ok(()),
        (Some(_), None) => Err("confirmed_project_root requires project_root_basis"),
        (None, Some(_)) => Err("project_root_basis requires confirmed_project_root"),
        (Some(_), Some(_)) => Err("Project root must be absolute and have an allowed basis"),
    }
}

fn current_host_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "posix"
    }
}

fn allowed_basis(value: &str) -> bool {
    matches!(
        value,
        "user_selected" | "controlled_fixture" | "confirmed_ableton_project_structure"
    )
}

fn availability_summary(candidates: &[crate::CandidatePathObservation]) -> String {
    if candidates
        .iter()
        .any(|candidate| candidate.availability_status == "existing_regular_file")
    {
        return "regular_file_observed".to_string();
    }
    if candidates.is_empty() {
        return "unknown".to_string();
    }
    if candidates
        .iter()
        .all(|candidate| candidate.platform_status == "foreign_platform_path")
    {
        return "unsupported_on_current_platform".to_string();
    }
    if candidates.iter().any(|candidate| {
        matches!(
            candidate.availability_status.as_str(),
            "inaccessible" | "not_checked"
        )
    }) {
        return "unknown".to_string();
    }
    "no_regular_file_observed".to_string()
}

fn complete_result(
    extraction: &DependencyExtractionResult,
    context: &PathObservationContext,
    dependency_observations: Vec<DependencyPathObservation>,
    warnings: Vec<PathObservationWarning>,
) -> PathObservationResult {
    let candidate_count = dependency_observations
        .iter()
        .map(|item| item.candidates.len())
        .sum();
    let regular_file_count = count_status(&dependency_observations, "existing_regular_file");
    let missing_count = count_status(&dependency_observations, "missing");
    let unknown_count = candidate_count - regular_file_count - missing_count;
    PathObservationResult {
        observation_metadata: metadata(
            extraction,
            context,
            candidate_count,
            regular_file_count,
            missing_count,
            unknown_count,
            warnings.len(),
            0,
        ),
        dependency_observations,
        warnings,
        errors: Vec::new(),
    }
}

fn count_status(observations: &[DependencyPathObservation], status: &str) -> usize {
    observations
        .iter()
        .flat_map(|item| &item.candidates)
        .filter(|candidate| candidate.availability_status == status)
        .count()
}

#[allow(clippy::too_many_arguments)]
fn metadata(
    extraction: &DependencyExtractionResult,
    context: &PathObservationContext,
    candidate_count: usize,
    regular_file_count: usize,
    missing_count: usize,
    unknown_count: usize,
    warning_count: usize,
    error_count: usize,
) -> PathObservationMetadata {
    PathObservationMetadata {
        observer_version: PATH_OBSERVER_VERSION.to_string(),
        path_observation_model_version: PATH_OBSERVATION_MODEL_VERSION.to_string(),
        input_dependency_ref_version: extraction
            .extraction_metadata
            .dependency_ref_version
            .clone(),
        source_als_path: extraction.extraction_metadata.source_als_path.clone(),
        source_file_hash: extraction.extraction_metadata.source_file_hash.clone(),
        project_root_basis: context.project_root_basis.clone(),
        dependency_count: extraction.dependencies.len(),
        candidate_count,
        regular_file_count,
        missing_count,
        unknown_count,
        warning_count,
        error_count,
    }
}

fn fatal_result(
    extraction: &DependencyExtractionResult,
    context: &PathObservationContext,
    code: &str,
    message: &str,
) -> PathObservationResult {
    let error = PathObservationError {
        error_code: code.to_string(),
        message: message.to_string(),
        input_dependency_ref_version: extraction
            .extraction_metadata
            .dependency_ref_version
            .clone(),
    };
    PathObservationResult {
        observation_metadata: metadata(extraction, context, 0, 0, 0, 0, 0, 1),
        dependency_observations: Vec::new(),
        warnings: Vec::new(),
        errors: vec![error],
    }
}
