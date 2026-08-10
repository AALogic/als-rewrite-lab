use crate::{
    DesktopApplicationError, DesktopCopyPreview, DesktopCopyResult, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest, ProjectSelection,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BATCH_COPY_SERVICE_VERSION: &str = "0.1.0";
pub const BATCH_COPY_DIAGNOSTIC_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPrepareCopyRequest {
    pub request_id: String,
    pub selections: Vec<ProjectSelection>,
    pub destination_parent: PathBuf,
    pub experimental_compatibility_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchCopyPreview {
    pub service_version: String,
    pub request_id: String,
    pub preview_status: String,
    pub destination_parent: PathBuf,
    pub jobs: Vec<BatchPreviewJob>,
    pub summary: BatchCopySummary,
    pub diagnostic_report: BatchCopyDiagnosticReport,
    pub errors: Vec<DesktopApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPreviewJob {
    pub job_id: String,
    pub selection_id: String,
    pub source_als_path: PathBuf,
    pub target_project_root: PathBuf,
    pub job_status: String,
    pub preview: Option<DesktopCopyPreview>,
    pub errors: Vec<DesktopApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchExecuteCopyRequest {
    pub request_id: String,
    pub preview: BatchCopyPreview,
    pub write_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchCopyResult {
    pub service_version: String,
    pub request_id: String,
    pub run_status: String,
    pub destination_parent: PathBuf,
    pub jobs: Vec<BatchCopyJobResult>,
    pub summary: BatchCopySummary,
    pub diagnostic_report: BatchCopyDiagnosticReport,
    pub errors: Vec<DesktopApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchCopyJobResult {
    pub job_id: String,
    pub selection_id: String,
    pub source_als_path: PathBuf,
    pub target_project_root: PathBuf,
    pub job_status: String,
    pub result: Option<DesktopCopyResult>,
    pub errors: Vec<DesktopApplicationError>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchCopySummary {
    pub total_job_count: usize,
    pub ready_job_count: usize,
    pub blocked_job_count: usize,
    pub completed_job_count: usize,
    pub incomplete_job_count: usize,
    pub failed_job_count: usize,
    pub cancelled_job_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchCopyDiagnosticReport {
    pub diagnostic_schema_version: String,
    pub request_id: String,
    pub service_version: String,
    pub host_os: String,
    pub host_arch: String,
    pub run_status: String,
    pub elapsed_ms: u64,
    pub summary: BatchCopySummary,
    pub error_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProgressEvent {
    pub request_id: String,
    pub stage: String,
    pub job_id: Option<String>,
    pub job_index: usize,
    pub total_job_count: usize,
    pub completed_job_count: usize,
    pub failed_job_count: usize,
}

pub trait BatchCopyObserver {
    fn on_progress(&mut self, _event: &BatchProgressEvent) {}
    fn is_cancelled(&self) -> bool {
        false
    }
}

impl BatchCopyObserver for () {}

pub trait BatchCopyOperations {
    fn prepare(&mut self, request: &DesktopPrepareCopyRequest) -> DesktopCopyPreview;
    fn execute(&mut self, request: &DesktopExecuteCopyRequest) -> DesktopCopyResult;
}

pub fn prepare_batch_copy(request: &BatchPrepareCopyRequest) -> BatchCopyPreview {
    let mut operations = crate::batch_copy_operations::DesktopCopyOperations;
    crate::batch_copy_prepare::prepare_batch_copy_impl(request, &mut operations)
}

#[doc(hidden)]
pub fn prepare_batch_copy_with_operations(
    request: &BatchPrepareCopyRequest,
    operations: &mut dyn BatchCopyOperations,
) -> BatchCopyPreview {
    crate::batch_copy_prepare::prepare_batch_copy_impl(request, operations)
}

pub fn execute_batch_copy(request: &BatchExecuteCopyRequest) -> BatchCopyResult {
    let mut operations = crate::batch_copy_operations::DesktopCopyOperations;
    let mut observer = ();
    crate::batch_copy_execute::execute_batch_copy_impl(request, &mut operations, &mut observer)
}

pub fn execute_batch_copy_controlled(
    request: &BatchExecuteCopyRequest,
    observer: &mut dyn BatchCopyObserver,
) -> BatchCopyResult {
    let mut operations = crate::batch_copy_operations::DesktopCopyOperations;
    crate::batch_copy_execute::execute_batch_copy_impl(request, &mut operations, observer)
}

#[doc(hidden)]
pub fn execute_batch_copy_with_operations(
    request: &BatchExecuteCopyRequest,
    operations: &mut dyn BatchCopyOperations,
    observer: &mut dyn BatchCopyObserver,
) -> BatchCopyResult {
    crate::batch_copy_execute::execute_batch_copy_impl(request, operations, observer)
}
