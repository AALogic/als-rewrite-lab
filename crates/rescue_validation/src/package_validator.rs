use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PACKAGE_VALIDATOR_VERSION: &str = "0.1.0";
pub const PACKAGE_VALIDATION_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationRequest {
    pub validation_id: String,
    pub staging_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationResult {
    pub metadata: PackageValidationMetadata,
    pub staging_root: PathBuf,
    pub final_target_root: PathBuf,
    pub file_records: Vec<FileValidationRecord>,
    pub semantic_diff_records: Vec<SemanticDiffRecord>,
    pub validation_status: String,
    pub warnings: Vec<PackageValidationWarning>,
    pub errors: Vec<PackageValidationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationMetadata {
    pub validator_version: String,
    pub validation_schema_version: String,
    pub validation_id: String,
    pub plan_id: String,
    pub execution_id: String,
    pub rewrite_id: String,
    pub source_als_hash: String,
    pub planned_file_count: usize,
    pub verified_file_count: usize,
    pub planned_rewrite_count: usize,
    pub verified_rewrite_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileValidationRecord {
    pub operation_id: String,
    pub target_relative_path: PathBuf,
    pub expected_sha256: String,
    pub observed_sha256: Option<String>,
    pub expected_size: Option<u64>,
    pub observed_size: Option<u64>,
    pub file_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiffRecord {
    pub operation_id: String,
    pub als_ref_id: usize,
    pub xml_locator: String,
    pub verified_fields: Vec<String>,
    pub diff_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationWarning {
    pub warning_code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageValidationError {
    pub error_code: String,
    pub message: String,
    pub operation_id: Option<String>,
    pub path: Option<PathBuf>,
}

pub fn validate_staged_package(
    request: &PackageValidationRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
) -> PackageValidationResult {
    crate::package_validator_impl::validate_staged_package_impl(request, plan, staging, rewrite)
}
