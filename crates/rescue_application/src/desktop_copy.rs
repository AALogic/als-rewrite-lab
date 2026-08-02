use crate::DesktopApplicationError;
use rescue_pipeline::PlanFingerprint;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DESKTOP_COPY_SERVICE_VERSION: &str = "0.4.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopPrepareCopyRequest {
    pub request_id: String,
    pub source_als_path: PathBuf,
    pub target_project_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopCopyPreview {
    pub service_version: String,
    pub request_id: String,
    pub preview_status: String,
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
