use crate::{DesktopApplicationError, DesktopCopyDiagnosticReport, DesktopDiagnosticReport};
use serde::{Deserialize, Serialize};

pub const DESKTOP_APPLICATION_PROFILE_SCHEMA_VERSION: &str = "0.1";
pub const COMPATIBILITY_TEST_REPORT_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopApplicationProfile {
    pub schema_version: String,
    pub application_profile: String,
    pub experimental_compatibility_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityTestReportRequest {
    pub analysis_report: DesktopDiagnosticReport,
    pub copy_report: Option<DesktopCopyDiagnosticReport>,
    pub manual_verification_outcome: String,
    pub tested_ableton_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityTestReport {
    pub report_schema_version: String,
    pub application_profile: String,
    pub build_commit: String,
    pub host_os: String,
    pub host_arch: String,
    pub analysis_report: DesktopDiagnosticReport,
    pub copy_report: Option<DesktopCopyDiagnosticReport>,
    pub manual_verification_outcome: String,
    pub tested_ableton_version: Option<String>,
    pub provisional_conclusion: String,
}

pub fn application_profile() -> DesktopApplicationProfile {
    let enabled = cfg!(feature = "compatibility-lab");
    DesktopApplicationProfile {
        schema_version: DESKTOP_APPLICATION_PROFILE_SCHEMA_VERSION.to_string(),
        application_profile: if enabled {
            "compatibility_lab"
        } else {
            "strict_alpha"
        }
        .to_string(),
        experimental_compatibility_available: enabled,
    }
}

pub fn finalize_compatibility_test_report(
    request: &CompatibilityTestReportRequest,
) -> Result<CompatibilityTestReport, DesktopApplicationError> {
    validate_manual_outcome(&request.manual_verification_outcome)?;
    let tested_version = sanitize_tested_version(request.tested_ableton_version.as_deref())?;
    let copy_report = request.copy_report.clone();
    let provisional_conclusion =
        conclusion(&request.manual_verification_outcome, copy_report.as_ref());

    Ok(CompatibilityTestReport {
        report_schema_version: COMPATIBILITY_TEST_REPORT_SCHEMA_VERSION.to_string(),
        application_profile: application_profile().application_profile,
        build_commit: request.analysis_report.build_commit.clone(),
        host_os: request.analysis_report.host_os.clone(),
        host_arch: request.analysis_report.host_arch.clone(),
        analysis_report: request.analysis_report.clone(),
        copy_report,
        manual_verification_outcome: request.manual_verification_outcome.clone(),
        tested_ableton_version: tested_version,
        provisional_conclusion: provisional_conclusion.to_string(),
    })
}

fn validate_manual_outcome(value: &str) -> Result<(), DesktopApplicationError> {
    if matches!(
        value,
        "not_checked"
            | "opened_without_missing_files"
            | "opened_with_missing_files"
            | "failed_to_open"
    ) {
        return Ok(());
    }
    Err(error(
        "COMPATIBILITY_MANUAL_OUTCOME_INVALID",
        "compatibility_report",
        "Manual Ableton verification outcome is unsupported",
    ))
}

fn sanitize_tested_version(value: Option<&str>) -> Result<Option<String>, DesktopApplicationError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > 80 || value.contains('/') || value.contains('\\') || value.contains(':') {
        return Err(error(
            "COMPATIBILITY_TESTED_VERSION_INVALID",
            "compatibility_report",
            "Tested Ableton version must be a short version label without a path",
        ));
    }
    Ok(Some(value.to_string()))
}

fn conclusion(
    manual_outcome: &str,
    copy_report: Option<&DesktopCopyDiagnosticReport>,
) -> &'static str {
    if copy_report.is_some_and(|report| !report.errors.is_empty())
        || matches!(
            manual_outcome,
            "opened_with_missing_files" | "failed_to_open"
        )
    {
        "candidate_failure"
    } else if manual_outcome == "opened_without_missing_files"
        && copy_report.is_some_and(|report| {
            matches!(
                report.run_status.as_str(),
                "complete_copy_ready_for_manual_check" | "incomplete_copy_ready_for_manual_check"
            )
        })
    {
        "candidate_success"
    } else {
        "insufficient_evidence"
    }
}

fn error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
