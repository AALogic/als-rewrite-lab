use rescue_packaging::PackagePlan;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const STAGING_EXECUTOR_VERSION: &str = "0.3.0";
pub const STAGING_EXECUTION_SCHEMA_VERSION: &str = "0.3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingExecutionRequest {
    pub execution_id: String,
    pub staging_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingExecutionResult {
    pub metadata: StagingExecutionMetadata,
    pub staging_root: PathBuf,
    pub staged_als_relative_path: PathBuf,
    pub directory_records: Vec<DirectoryExecutionRecord>,
    pub copy_records: Vec<CopyExecutionRecord>,
    pub execution_status: String,
    pub warnings: Vec<StagingExecutionWarning>,
    pub errors: Vec<StagingExecutionError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingExecutionMetadata {
    pub executor_version: String,
    pub execution_schema_version: String,
    pub execution_id: String,
    pub plan_id: String,
    pub source_als_hash: String,
    pub planned_directory_count: usize,
    pub completed_directory_count: usize,
    pub planned_copy_count: usize,
    pub completed_copy_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryExecutionRecord {
    pub operation_id: String,
    pub target_relative_path: PathBuf,
    pub purpose: String,
    pub operation_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyExecutionRecord {
    pub operation_id: String,
    pub operation_kind: String,
    pub source_path: PathBuf,
    pub target_relative_path: PathBuf,
    pub expected_sha256: Option<String>,
    pub observed_sha256: Option<String>,
    pub expected_size: u64,
    pub observed_size: Option<u64>,
    pub verification_method: String,
    pub operation_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingExecutionWarning {
    pub warning_code: String,
    pub message: String,
    pub operation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingExecutionError {
    pub error_code: String,
    pub message: String,
    pub operation_id: Option<String>,
    pub path: Option<PathBuf>,
}

pub fn execute_staging(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
) -> StagingExecutionResult {
    crate::staging_executor_impl::execute_staging_impl(request, plan)
}
