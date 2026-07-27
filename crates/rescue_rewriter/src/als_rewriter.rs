use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ALS_REWRITER_VERSION: &str = "0.1.0";
pub const ALS_REWRITE_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSRewriteRequest {
    pub rewrite_id: String,
    pub staging_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSRewriteResult {
    pub metadata: ALSRewriteMetadata,
    pub staged_als_relative_path: PathBuf,
    pub original_staged_als_hash: Option<String>,
    pub rewritten_staged_als_hash: Option<String>,
    pub operation_records: Vec<RewriteExecutionRecord>,
    pub rewrite_status: String,
    pub warnings: Vec<ALSRewriteWarning>,
    pub errors: Vec<ALSRewriteError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSRewriteMetadata {
    pub rewriter_version: String,
    pub rewrite_schema_version: String,
    pub rewrite_id: String,
    pub execution_id: String,
    pub plan_id: String,
    pub source_als_hash: String,
    pub rewrite_ruleset_version: String,
    pub planned_operation_count: usize,
    pub completed_operation_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteExecutionRecord {
    pub operation_id: String,
    pub als_ref_id: usize,
    pub xml_locator: String,
    pub changed_fields: Vec<String>,
    pub operation_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSRewriteWarning {
    pub warning_code: String,
    pub message: String,
    pub operation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSRewriteError {
    pub error_code: String,
    pub message: String,
    pub operation_id: Option<String>,
    pub path: Option<PathBuf>,
}

pub fn rewrite_staged_als(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
) -> ALSRewriteResult {
    crate::als_rewriter_impl::rewrite_staged_als_impl(request, plan, staging)
}
