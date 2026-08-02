use crate::{
    desktop_diagnostic, DesktopAnalyzeRequest, DesktopAnalyzeResult, DesktopApplicationError,
    DESKTOP_APPLICATION_SERVICE_VERSION,
};
use rescue_analyzer::{
    assess_dependencies, build_preflight_report, discover_project, ProjectDiscoveryRequest,
};
use rescue_core::{
    analyze_als, extract_dependencies, observe_dependency_paths, PathObservationContext,
};

pub(crate) fn analyze_project_impl(request: &DesktopAnalyzeRequest) -> DesktopAnalyzeResult {
    if let Some(error) = validate_request(request) {
        return failed_result(request, vec![error]);
    }
    let discovery = discover_project(&ProjectDiscoveryRequest {
        source_als_path: request.source_als_path.clone(),
    });
    if !discovery.errors.is_empty() {
        let errors = discovery
            .errors
            .iter()
            .map(|error| DesktopApplicationError {
                error_code: error.error_code.clone(),
                stage: "project_discovery".to_string(),
                message: error.message.clone(),
            })
            .collect();
        return failed_result(request, errors);
    }
    let analysis = match analyze_als(&request.source_als_path) {
        Ok(analysis) => analysis,
        Err(error) => {
            let info = error.to_info();
            return failed_result(
                request,
                vec![DesktopApplicationError {
                    error_code: info.error_code,
                    stage: "als_reader".to_string(),
                    message: info.message,
                }],
            );
        }
    };
    let extraction = extract_dependencies(&analysis);
    let observations = observe_dependency_paths(
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
    let assessment = assess_dependencies(&extraction, &observations);
    let report = build_preflight_report(&discovery, &assessment);
    if !report.errors.is_empty() {
        let errors = report
            .errors
            .iter()
            .map(|error| DesktopApplicationError {
                error_code: error.error_code.clone(),
                stage: "preflight_report".to_string(),
                message: error.message.clone(),
            })
            .collect();
        return result_with_report(request, "analysis_failed", report, errors);
    }
    result_with_report(request, "analysis_complete", report, Vec::new())
}

fn validate_request(request: &DesktopAnalyzeRequest) -> Option<DesktopApplicationError> {
    if request.request_id.trim().is_empty() {
        return Some(error(
            "DESKTOP_REQUEST_ID_EMPTY",
            "request_validation",
            "Request ID must not be empty",
        ));
    }
    if !request.source_als_path.is_absolute() {
        return Some(error(
            "DESKTOP_SOURCE_PATH_NOT_ABSOLUTE",
            "request_validation",
            "Selected ALS path must be absolute",
        ));
    }
    None
}

fn result_with_report(
    request: &DesktopAnalyzeRequest,
    status: &str,
    report: rescue_analyzer::PreflightReport,
    errors: Vec<DesktopApplicationError>,
) -> DesktopAnalyzeResult {
    let diagnostic = desktop_diagnostic::from_preflight(request, status, &report, &errors);
    DesktopAnalyzeResult {
        service_version: DESKTOP_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: status.to_string(),
        preflight_report: Some(report),
        diagnostic_report: diagnostic,
        errors,
    }
}

fn failed_result(
    request: &DesktopAnalyzeRequest,
    errors: Vec<DesktopApplicationError>,
) -> DesktopAnalyzeResult {
    DesktopAnalyzeResult {
        service_version: DESKTOP_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: "analysis_failed".to_string(),
        preflight_report: None,
        diagnostic_report: desktop_diagnostic::from_errors(request, &errors),
        errors,
    }
}

fn error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
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
