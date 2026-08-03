use crate::{
    desktop_diagnostic, DesktopAnalyzeRequest, DesktopAnalyzeResult, DesktopApplicationError,
    DESKTOP_APPLICATION_SERVICE_VERSION,
};
use rescue_analyzer::{
    assess_dependencies, build_preflight_report, discover_project, ProjectDiscoveryRequest,
};
use rescue_core::{
    analyze_als, assess_rewrite_compatibility, extract_dependencies, observe_dependency_paths,
    PathObservationContext, RewriteCompatibilityAssessment,
};
use std::time::Instant;

pub(crate) fn analyze_project_impl(request: &DesktopAnalyzeRequest) -> DesktopAnalyzeResult {
    let started = Instant::now();
    if let Some(error) = validate_request(request) {
        return failed_result(request, vec![error], elapsed_ms(started));
    }
    let discovery = discover_project(&ProjectDiscoveryRequest {
        source_als_path: request.source_als_path.clone(),
    });
    if !discovery.errors.is_empty() {
        let errors = discovery_errors(&discovery.errors);
        return failed_result(request, errors, elapsed_ms(started));
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
                elapsed_ms(started),
            );
        }
    };
    let compatibility = assess_rewrite_compatibility(&analysis);
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
        let errors = preflight_errors(&report.errors);
        return result_with_report(
            request,
            "analysis_failed",
            report,
            compatibility,
            errors,
            elapsed_ms(started),
        );
    }
    result_with_report(
        request,
        "analysis_complete",
        report,
        compatibility,
        Vec::new(),
        elapsed_ms(started),
    )
}

fn discovery_errors(
    errors: &[rescue_analyzer::ProjectDiscoveryError],
) -> Vec<DesktopApplicationError> {
    errors
        .iter()
        .map(|error| DesktopApplicationError {
            error_code: error.error_code.clone(),
            stage: "project_discovery".to_string(),
            message: error.message.clone(),
        })
        .collect()
}

fn preflight_errors(
    errors: &[rescue_analyzer::PreflightReportError],
) -> Vec<DesktopApplicationError> {
    errors
        .iter()
        .map(|error| DesktopApplicationError {
            error_code: error.error_code.clone(),
            stage: "preflight_report".to_string(),
            message: error.message.clone(),
        })
        .collect()
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
    compatibility: RewriteCompatibilityAssessment,
    errors: Vec<DesktopApplicationError>,
    elapsed_ms: u64,
) -> DesktopAnalyzeResult {
    let diagnostic = desktop_diagnostic::from_preflight(
        request,
        status,
        &report,
        compatibility,
        &errors,
        elapsed_ms,
    );
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
    elapsed_ms: u64,
) -> DesktopAnalyzeResult {
    DesktopAnalyzeResult {
        service_version: DESKTOP_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: "analysis_failed".to_string(),
        preflight_report: None,
        diagnostic_report: desktop_diagnostic::from_errors(request, &errors, elapsed_ms),
        errors,
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
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
