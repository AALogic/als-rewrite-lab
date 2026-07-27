use crate::{ALSRewriteError, ALSRewriteRequest};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::{PackagePlan, RewriteOperation};
use std::collections::BTreeSet;
use std::path::{Component, Path};

pub(crate) fn validate_inputs(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
) -> Vec<ALSRewriteError> {
    let mut errors = Vec::new();
    if plan.plan_status != "ready_for_laboratory_execution"
        || !plan.errors.is_empty()
        || !plan.unresolved_requirements.is_empty()
    {
        errors.push(error(
            "REWRITE_PLAN_NOT_READY",
            "Only a complete laboratory rewrite plan may be executed",
            None,
            None,
        ));
    }
    if staging.execution_status != "staging_complete" || !staging.errors.is_empty() {
        errors.push(error(
            "REWRITE_STAGING_NOT_COMPLETE",
            "Rewrite requires a completed staging execution",
            None,
            Some(&staging.staging_root),
        ));
    }
    if request.staging_root != staging.staging_root {
        errors.push(error(
            "REWRITE_STAGING_ROOT_MISMATCH",
            "Request and staging result identify different roots",
            None,
            Some(&request.staging_root),
        ));
    }
    if staging.metadata.plan_id != plan.metadata.plan_id
        || staging.metadata.source_als_hash != plan.metadata.source_als_hash
    {
        errors.push(error(
            "REWRITE_STAGING_PLAN_MISMATCH",
            "Staging result does not belong to the supplied plan",
            None,
            None,
        ));
    }
    if !safe_relative(&staging.staged_als_relative_path) {
        errors.push(error(
            "REWRITE_STAGED_ALS_PATH_UNSAFE",
            "Staged ALS path must be a safe relative path",
            None,
            Some(&staging.staged_als_relative_path),
        ));
    }
    validate_operations(plan, &mut errors);
    errors
}

fn validate_operations(plan: &PackagePlan, errors: &mut Vec<ALSRewriteError>) {
    let expected_fields = ["Path", "RelativePath", "RelativePathType"];
    let copy_targets: BTreeSet<_> = plan
        .copy_operations
        .iter()
        .map(|operation| operation.target_relative_path.as_path())
        .collect();
    for operation in &plan.rewrite_operations {
        if operation.source_als_hash != plan.source_als.source_file_hash {
            errors.push(operation_error(
                "REWRITE_OPERATION_SNAPSHOT_MISMATCH",
                "Rewrite operation belongs to another ALS snapshot",
                operation,
            ));
        }
        if operation.rule_id != plan.metadata.rewrite_ruleset_version
            || operation.support_status != "experimental_lab_only"
        {
            errors.push(operation_error(
                "REWRITE_RULE_UNSUPPORTED",
                "Rewrite operation is outside the accepted laboratory ruleset",
                operation,
            ));
        }
        if operation.fields_to_change
            != expected_fields
                .iter()
                .map(|field| field.to_string())
                .collect::<Vec<_>>()
            || operation.new_relative_path_type != "3"
        {
            errors.push(operation_error(
                "REWRITE_FIELDS_UNSUPPORTED",
                "Only the three E-03 path fields may change",
                operation,
            ));
        }
        let relative = Path::new(&operation.new_relative_path);
        let expected_absolute = plan.target_project_root.join(relative);
        if !safe_relative(relative)
            || Path::new(&operation.new_path) != expected_absolute
            || !copy_targets.contains(relative)
        {
            errors.push(operation_error(
                "REWRITE_TARGET_CONTRACT_MISMATCH",
                "Rewrite target must address a planned copied asset",
                operation,
            ));
        }
    }
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn operation_error(code: &str, message: &str, operation: &RewriteOperation) -> ALSRewriteError {
    error(code, message, Some(&operation.operation_id), None)
}

fn error(
    code: &str,
    message: &str,
    operation_id: Option<&str>,
    path: Option<&Path>,
) -> ALSRewriteError {
    crate::als_rewriter_result::rewrite_error(
        code,
        message.to_string(),
        operation_id.map(ToString::to_string),
        path.map(Path::to_path_buf),
    )
}
