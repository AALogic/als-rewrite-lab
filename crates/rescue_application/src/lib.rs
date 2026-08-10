mod batch_copy;
mod batch_copy_execute;
mod batch_copy_operations;
mod batch_copy_prepare;
mod batch_copy_result;
mod batch_copy_validation;
mod collection_delivery;
mod collection_delivery_io;
mod compatibility_report;
mod courier_collection;
mod courier_collection_result;
mod courier_work_queue;
mod desktop_analysis;
mod desktop_copy;
mod desktop_copy_diagnostic;
mod desktop_copy_impl;
mod desktop_copy_result;
mod desktop_diagnostic;
mod project_catalog_application;
mod project_catalog_groups;
mod project_catalog_projection;
mod project_catalog_refresh;
mod project_selection;

pub use batch_copy::{
    execute_batch_copy, execute_batch_copy_controlled, execute_batch_copy_with_operations,
    prepare_batch_copy, prepare_batch_copy_with_operations, BatchCopyDiagnosticReport,
    BatchCopyJobResult, BatchCopyObserver, BatchCopyOperations, BatchCopyPreview, BatchCopyResult,
    BatchCopySummary, BatchExecuteCopyRequest, BatchPrepareCopyRequest, BatchPreviewJob,
    BatchProgressEvent, BATCH_COPY_DIAGNOSTIC_SCHEMA_VERSION, BATCH_COPY_SERVICE_VERSION,
};
pub(crate) use collection_delivery::delivery_error;
pub use collection_delivery::{
    execute_collection_delivery, plan_collection_delivery, CollectionDeliveryError,
    CollectionDeliveryItemResult, CollectionDeliveryOperation, CollectionDeliveryPlan,
    CollectionDeliveryRequest, CollectionDeliveryResult, COLLECTION_DELIVERY_SCHEMA_VERSION,
};
pub use compatibility_report::{
    application_profile, finalize_compatibility_test_report, CompatibilityTestReport,
    CompatibilityTestReportRequest, DesktopApplicationProfile,
    COMPATIBILITY_TEST_REPORT_SCHEMA_VERSION, DESKTOP_APPLICATION_PROFILE_SCHEMA_VERSION,
};
pub use courier_collection::{
    CourierBatchWave, CourierCollectionError, CourierCollectionItem, CourierCollectionOrchestrator,
    CourierCollectionSnapshot, CourierHandoffSnapshot, DeliveryCollectionSnapshot,
    COURIER_COLLECTION_SCHEMA_VERSION,
};
pub use courier_work_queue::{
    CourierQueueError, CourierQueueSnapshot, CourierWorkItem, CourierWorkQueue,
    COURIER_QUEUE_SCHEMA_VERSION,
};
pub use desktop_copy::{
    default_target_project_root, execute_copy, prepare_copy, DesktopCopyDiagnosticReport,
    DesktopCopyPreview, DesktopCopyResult, DesktopDiagnosticError, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest, DESKTOP_COPY_DIAGNOSTIC_SCHEMA_VERSION,
    DESKTOP_COPY_SERVICE_VERSION,
};
pub use project_catalog_application::{
    list_project_catalog, refresh_project_catalog, resolve_project_selection,
    ProjectCatalogApplicationError, ProjectCatalogApplicationWarning, ProjectCatalogListMetadata,
    ProjectCatalogListRequest, ProjectCatalogListResult, ProjectCatalogRefreshRequest,
    ProjectCatalogRefreshResult, ProjectListGroup, ProjectListItem, ProjectSelection,
    ProjectSelectionRequest, ProjectSelectionResult, PROJECT_CATALOG_APPLICATION_SERVICE_VERSION,
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
