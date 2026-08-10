use crate::DesktopApplicationError;
use rescue_pipeline::PlanFingerprint;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DESKTOP_COPY_SERVICE_VERSION: &str = "0.6.0";
pub const DESKTOP_COPY_DIAGNOSTIC_SCHEMA_VERSION: &str = "0.2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopDiagnosticError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCopyDiagnosticReport {
    pub diagnostic_schema_version: String,
    pub request_id: String,
    pub operation_kind: String,
    pub service_version: String,
    pub pipeline_version: String,
    pub build_commit: String,
    pub host_os: String,
    pub host_arch: String,
    pub rewrite_policy: String,
    pub ableton_document_version: Option<String>,
    pub ableton_creator_version: Option<String>,
    pub ableton_minor_version: Option<String>,
    pub compatibility_status: String,
    pub run_status: String,
    pub completed_stage: String,
    pub elapsed_ms: u64,
    pub required_asset_count: usize,
    pub system_dependency_count: usize,
    pub copied_asset_count: usize,
    pub rewritten_reference_count: usize,
    pub omitted_asset_count: usize,
    pub errors: Vec<DesktopDiagnosticError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopPrepareCopyRequest {
    pub request_id: String,
    pub source_als_path: PathBuf,
    pub target_project_root: PathBuf,
    pub experimental_compatibility_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCopyPreview {
    pub service_version: String,
    pub request_id: String,
    pub preview_status: String,
    pub rewrite_policy: String,
    pub source_als_path: PathBuf,
    pub source_als_sha256: String,
    pub plan_fingerprint: Option<PlanFingerprint>,
    pub target_project_root: PathBuf,
    pub required_asset_count: usize,
    pub system_dependency_count: usize,
    pub copy_asset_count: usize,
    pub rewrite_reference_count: usize,
    pub omitted_asset_count: usize,
    pub expected_result_status: String,
    pub diagnostic_report: DesktopCopyDiagnosticReport,
    pub errors: Vec<DesktopApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopExecuteCopyRequest {
    pub request_id: String,
    pub preview: DesktopCopyPreview,
    pub write_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCopyResult {
    pub service_version: String,
    pub request_id: String,
    pub run_status: String,
    pub final_target_root: Option<PathBuf>,
    pub system_dependency_count: usize,
    pub copied_asset_count: usize,
    pub rewritten_reference_count: usize,
    pub omitted_asset_count: usize,
    pub diagnostic_report: DesktopCopyDiagnosticReport,
    pub errors: Vec<DesktopApplicationError>,
}

pub fn default_target_project_root(
    source_als_path: &Path,
    destination_parent: &Path,
) -> Result<PathBuf, DesktopApplicationError> {
    crate::desktop_copy_impl::default_target(source_als_path, destination_parent)
}

pub fn prepare_copy(request: &DesktopPrepareCopyRequest) -> DesktopCopyPreview {
    crate::desktop_copy_impl::prepare(request)
}

pub fn execute_copy(request: &DesktopExecuteCopyRequest) -> DesktopCopyResult {
    crate::desktop_copy_impl::execute(request)
}
