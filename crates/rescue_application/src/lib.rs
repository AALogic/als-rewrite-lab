mod compatibility_report;
mod desktop_analysis;
mod desktop_copy;
mod desktop_copy_diagnostic;
mod desktop_copy_impl;
mod desktop_copy_result;
mod desktop_diagnostic;

pub use compatibility_report::{
    application_profile, finalize_compatibility_test_report, CompatibilityTestReport,
    CompatibilityTestReportRequest, DesktopApplicationProfile,
    COMPATIBILITY_TEST_REPORT_SCHEMA_VERSION, DESKTOP_APPLICATION_PROFILE_SCHEMA_VERSION,
};
pub use desktop_copy::{
    default_target_project_root, execute_copy, prepare_copy, DesktopCopyDiagnosticReport,
    DesktopCopyPreview, DesktopCopyResult, DesktopDiagnosticError, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest, DESKTOP_COPY_DIAGNOSTIC_SCHEMA_VERSION,
    DESKTOP_COPY_SERVICE_VERSION,
};

use rescue_analyzer::{PreflightReport, PreflightSummary};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DESKTOP_APPLICATION_SERVICE_VERSION: &str = "0.3.0";
pub const DESKTOP_DIAGNOSTIC_SCHEMA_VERSION: &str = "0.3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopAnalyzeRequest {
    pub request_id: String,
    pub source_als_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopAnalyzeResult {
    pub service_version: String,
    pub request_id: String,
    pub run_status: String,
    pub preflight_report: Option<PreflightReport>,
    pub diagnostic_report: DesktopDiagnosticReport,
    pub errors: Vec<DesktopApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopDiagnosticReport {
    pub diagnostic_schema_version: String,
    pub request_id: String,
    pub service_version: String,
    pub build_commit: String,
    pub host_os: String,
    pub host_arch: String,
    pub run_status: String,
    pub elapsed_ms: u64,
    pub source_als_sha256: Option<String>,
    pub summary: Option<PreflightSummary>,
    pub requirements: Vec<DiagnosticRequirement>,
    pub rewrite_compatibility: Option<rescue_core::RewriteCompatibilityAssessment>,
    pub error_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRequirement {
    pub required_asset_id: String,
    pub occurrence_count: usize,
    pub source_category: String,
    pub management_class: String,
    pub portability_status: String,
    pub availability_status: String,
    pub resolution_status: String,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopApplicationError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

pub fn analyze_project(request: &DesktopAnalyzeRequest) -> DesktopAnalyzeResult {
    desktop_analysis::analyze_project_impl(request)
}
