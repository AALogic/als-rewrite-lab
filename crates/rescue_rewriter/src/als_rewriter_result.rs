use crate::{
    ALSRewriteError, ALSRewriteMetadata, ALSRewriteRequest, ALSRewriteResult,
    RewriteExecutionRecord, ALS_REWRITER_VERSION, ALS_REWRITE_SCHEMA_VERSION,
};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::{PackagePlan, RewriteOperation};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(crate) fn records_for(
    operations: &[RewriteOperation],
    completed_ids: &[String],
) -> Vec<RewriteExecutionRecord> {
    let completed: BTreeSet<_> = completed_ids.iter().map(String::as_str).collect();
    operations
        .iter()
        .map(|operation| RewriteExecutionRecord {
            operation_id: operation.operation_id.clone(),
            als_ref_id: operation.als_ref_id,
            xml_locator: operation.xml_locator.clone(),
            changed_fields: operation.fields_to_change.clone(),
            operation_status: if completed.contains(operation.operation_id.as_str()) {
                "rewritten_and_verified"
            } else {
                "failed"
            }
            .to_string(),
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn result(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    original_hash: Option<String>,
    rewritten_hash: Option<String>,
    records: Vec<RewriteExecutionRecord>,
    status: &str,
    errors: Vec<ALSRewriteError>,
) -> ALSRewriteResult {
    let completed = records
        .iter()
        .filter(|record| record.operation_status == "rewritten_and_verified")
        .count();
    ALSRewriteResult {
        metadata: ALSRewriteMetadata {
            rewriter_version: ALS_REWRITER_VERSION.to_string(),
            rewrite_schema_version: ALS_REWRITE_SCHEMA_VERSION.to_string(),
            rewrite_id: request.rewrite_id.clone(),
            execution_id: staging.metadata.execution_id.clone(),
            plan_id: plan.metadata.plan_id.clone(),
            source_als_hash: plan.metadata.source_als_hash.clone(),
            rewrite_ruleset_version: plan.metadata.rewrite_ruleset_version.clone(),
            planned_operation_count: plan.rewrite_operations.len(),
            completed_operation_count: completed,
            warning_count: 0,
            error_count: errors.len(),
        },
        staged_als_relative_path: staging.staged_als_relative_path.clone(),
        original_staged_als_hash: original_hash,
        rewritten_staged_als_hash: rewritten_hash,
        operation_records: records,
        rewrite_status: status.to_string(),
        warnings: Vec::new(),
        errors,
    }
}

pub(crate) fn rewrite_error(
    code: &str,
    message: String,
    operation_id: Option<String>,
    path: Option<PathBuf>,
) -> ALSRewriteError {
    ALSRewriteError {
        error_code: code.to_string(),
        message,
        operation_id,
        path,
    }
}
