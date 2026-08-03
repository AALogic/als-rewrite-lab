use crate::{
    DesktopAnalyzeRequest, DesktopApplicationError, DesktopDiagnosticReport, DiagnosticRequirement,
    DESKTOP_APPLICATION_SERVICE_VERSION, DESKTOP_DIAGNOSTIC_SCHEMA_VERSION,
};
use rescue_analyzer::PreflightReport;
use rescue_core::RewriteCompatibilityAssessment;

pub(crate) fn from_preflight(
    request: &DesktopAnalyzeRequest,
    run_status: &str,
    report: &PreflightReport,
    compatibility: RewriteCompatibilityAssessment,
    errors: &[DesktopApplicationError],
    elapsed_ms: u64,
) -> DesktopDiagnosticReport {
    DesktopDiagnosticReport {
        diagnostic_schema_version: DESKTOP_DIAGNOSTIC_SCHEMA_VERSION.to_string(),
        request_id: request.request_id.clone(),
        service_version: DESKTOP_APPLICATION_SERVICE_VERSION.to_string(),
        build_commit: build_commit(),
        host_os: std::env::consts::OS.to_string(),
        host_arch: std::env::consts::ARCH.to_string(),
        run_status: run_status.to_string(),
        elapsed_ms,
        source_als_sha256: Some(report.report_metadata.source_file_hash.clone()),
        summary: Some(report.summary.clone()),
        requirements: report
            .requirements
            .iter()
            .map(|requirement| DiagnosticRequirement {
                required_asset_id: requirement.required_asset_id.clone(),
                occurrence_count: requirement.occurrence_count,
                source_category: requirement.source_category.clone(),
                management_class: requirement.management_class.clone(),
                portability_status: requirement.portability_status.clone(),
                availability_status: requirement.availability_status.clone(),
                resolution_status: requirement.resolution_status.clone(),
                risk_flags: requirement.risk_flags.clone(),
            })
            .collect(),
        rewrite_compatibility: Some(compatibility),
        error_codes: error_codes(errors),
    }
}

pub(crate) fn from_errors(
    request: &DesktopAnalyzeRequest,
    errors: &[DesktopApplicationError],
    elapsed_ms: u64,
) -> DesktopDiagnosticReport {
    DesktopDiagnosticReport {
        diagnostic_schema_version: DESKTOP_DIAGNOSTIC_SCHEMA_VERSION.to_string(),
        request_id: request.request_id.clone(),
        service_version: DESKTOP_APPLICATION_SERVICE_VERSION.to_string(),
        build_commit: build_commit(),
        host_os: std::env::consts::OS.to_string(),
        host_arch: std::env::consts::ARCH.to_string(),
        run_status: "analysis_failed".to_string(),
        elapsed_ms,
        source_als_sha256: None,
        summary: None,
        requirements: Vec::new(),
        rewrite_compatibility: None,
        error_codes: error_codes(errors),
    }
}

fn build_commit() -> String {
    option_env!("ALS_RESCUE_BUILD_COMMIT")
        .unwrap_or("development")
        .to_string()
}

fn error_codes(errors: &[DesktopApplicationError]) -> Vec<String> {
    errors
        .iter()
        .map(|error| error.error_code.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::from_preflight;
    use crate::DesktopAnalyzeRequest;
    use rescue_analyzer::{
        PreflightProjectContext, PreflightReport, PreflightReportMetadata, PreflightRequirement,
        PreflightSummary,
    };
    use rescue_core::RewriteCompatibilityAssessment;
    use std::path::PathBuf;

    #[test]
    fn diagnostic_preserves_system_dependency_classification() {
        let request = DesktopAnalyzeRequest {
            request_id: "diagnostic-system-dependency".to_string(),
            source_als_path: PathBuf::from("/private/source.als"),
        };
        let report = PreflightReport {
            report_metadata: PreflightReportMetadata {
                report_version: "0.2.0".to_string(),
                assessment_version: "0.2.0".to_string(),
                source_als_path: "/private/source.als".to_string(),
                source_file_hash: "als-hash".to_string(),
                requirement_count: 1,
                notice_count: 0,
                error_count: 0,
            },
            project: PreflightProjectContext {
                discovery_status: "confirmed_project_root".to_string(),
                confirmed_project_root: Some("/private".to_string()),
                set_location: "project_root".to_string(),
                candidate_root_count: 1,
            },
            summary: PreflightSummary {
                overall_status: "ready".to_string(),
                reference_occurrence_count: 1,
                required_asset_count: 1,
                system_dependency_count: 1,
                candidate_observed_count: 1,
                needs_search_count: 0,
                unknown_count: 0,
                unresolved_count: 0,
            },
            requirements: vec![PreflightRequirement {
                required_asset_id: "required_asset_000000".to_string(),
                filename: Some("private.wav".to_string()),
                occurrence_count: 1,
                source_category: "ableton_core_library".to_string(),
                management_class: "system_dependency".to_string(),
                portability_status: "portable_risk".to_string(),
                availability_status: "available_at_recorded_path".to_string(),
                resolution_status: "not_required".to_string(),
                candidate_paths: vec!["/private/private.wav".to_string()],
                risk_flags: vec!["portable_risk".to_string()],
            }],
            notices: Vec::new(),
            errors: Vec::new(),
        };

        let diagnostic = from_preflight(
            &request,
            "analysis_complete",
            &report,
            compatibility(),
            &[],
            1,
        );
        let Some(summary) = diagnostic.summary.as_ref() else {
            panic!("diagnostic summary is required");
        };
        let requirement = &diagnostic.requirements[0];

        assert_eq!(summary.system_dependency_count, 1);
        assert_eq!(requirement.source_category, "ableton_core_library");
        assert_eq!(requirement.management_class, "system_dependency");
        assert_eq!(requirement.portability_status, "portable_risk");
    }

    fn compatibility() -> RewriteCompatibilityAssessment {
        RewriteCompatibilityAssessment {
            schema_version: "0.1".to_string(),
            document_profile: "confirmed_live_11_3".to_string(),
            evidence_status: "confirmed_profile".to_string(),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
            ableton_schema_change_count: Some("7".to_string()),
            active_reference_count: 1,
            strict_supported_count: 1,
            lab_compatible_count: 1,
            unsupported_shape_count: 0,
            references: Vec::new(),
        }
    }
}
