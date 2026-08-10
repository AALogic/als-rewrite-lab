use crate::{LaboratoryPackageError, LaboratoryPackageRequest};
use rescue_analyzer::{
    assess_dependencies, build_preflight_report, discover_project, DependencyAssessmentResult,
    PreflightReport, ProjectDiscoveryRequest, ProjectDiscoveryResult,
};
use rescue_core::{
    analyze_als, extract_dependencies, observe_dependency_paths, ALSReadModel,
    DependencyExtractionResult, PathObservationContext, PathObservationResult,
};

pub(crate) struct ReadStage {
    pub discovery: ProjectDiscoveryResult,
    pub als_read_model: ALSReadModel,
    pub extraction: DependencyExtractionResult,
    pub path_observations: PathObservationResult,
    pub assessment: DependencyAssessmentResult,
    pub preflight: PreflightReport,
}

pub(crate) struct ReadFailure {
    pub discovery: ProjectDiscoveryResult,
    pub error: LaboratoryPackageError,
}

// A blocked run retains the complete discovery evidence for its public result.
#[allow(clippy::result_large_err)]
pub(crate) fn read_and_assess(
    request: &LaboratoryPackageRequest,
) -> Result<ReadStage, ReadFailure> {
    let discovery = discover_project(&ProjectDiscoveryRequest {
        source_als_path: request.source_als_path.clone(),
    });
    if !discovery.errors.is_empty() {
        return Err(failure(
            &discovery,
            error(
                "PIPELINE_PROJECT_DISCOVERY_FAILED",
                "project_discovery",
                "Project discovery returned errors",
            ),
        ));
    }
    if let Some(error) =
        crate::laboratory_pipeline_inputs::validate_discovered_project(request, &discovery)
    {
        return Err(failure(&discovery, error));
    }
    let als_read_model = analyze_als(&request.source_als_path).map_err(|read_error| {
        let info = read_error.to_info();
        failure(
            &discovery,
            error(&info.error_code, "als_reader", &info.message),
        )
    })?;
    let extraction = extract_dependencies(&als_read_model);
    if !extraction.errors.is_empty() {
        return Err(failure(
            &discovery,
            error(
                "PIPELINE_DEPENDENCY_EXTRACTION_FAILED",
                "dependency_extraction",
                "Dependency extraction returned errors",
            ),
        ));
    }
    let path_observations = observe_dependency_paths(
        &extraction,
        &PathObservationContext {
            host_platform: current_host_platform().to_string(),
            confirmed_project_root: discovery.confirmed_project_root.clone(),
            project_root_basis: discovery
                .confirmed_project_root
                .as_ref()
                .map(|_| "confirmed_ableton_project_structure".to_string()),
        },
    );
    if !path_observations.errors.is_empty() {
        return Err(failure(
            &discovery,
            error(
                "PIPELINE_PATH_OBSERVATION_FAILED",
                "path_observation",
                "Path observation returned errors",
            ),
        ));
    }
    let assessment = assess_dependencies(&extraction, &path_observations);
    if !assessment.errors.is_empty() {
        return Err(failure(
            &discovery,
            error(
                "PIPELINE_DEPENDENCY_ASSESSMENT_FAILED",
                "dependency_assessment",
                "Dependency assessment returned errors",
            ),
        ));
    }
    let preflight = build_preflight_report(&discovery, &assessment);
    if !preflight.errors.is_empty() {
        return Err(failure(
            &discovery,
            error(
                "PIPELINE_PREFLIGHT_FAILED",
                "preflight",
                "Preflight report returned errors",
            ),
        ));
    }
    Ok(ReadStage {
        discovery,
        als_read_model,
        extraction,
        path_observations,
        assessment,
        preflight,
    })
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

fn failure(discovery: &ProjectDiscoveryResult, error: LaboratoryPackageError) -> ReadFailure {
    ReadFailure {
        discovery: discovery.clone(),
        error,
    }
}

fn error(code: &str, stage: &str, message: &str) -> LaboratoryPackageError {
    LaboratoryPackageError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
