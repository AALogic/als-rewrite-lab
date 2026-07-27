use crate::{PackageValidationRequest, PackageValidationResult};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;

pub(crate) fn validate_staged_package_impl(
    request: &PackageValidationRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
) -> PackageValidationResult {
    let input_errors =
        crate::package_validator_inputs::validate_inputs(request, plan, staging, rewrite);
    if !input_errors.is_empty() {
        return crate::package_validator_result::result(
            request,
            plan,
            staging,
            rewrite,
            Vec::new(),
            Vec::new(),
            input_errors,
        );
    }
    let file_outcome =
        crate::package_validator_files::validate_files(&request.staging_root, plan, rewrite);
    let staged_als = request.staging_root.join(&staging.staged_als_relative_path);
    let semantic_outcome = crate::semantic_diff::validate_semantic_diff(plan, &staged_als, rewrite);
    let mut errors = file_outcome.errors;
    errors.extend(semantic_outcome.errors);
    crate::package_validator_result::result(
        request,
        plan,
        staging,
        rewrite,
        file_outcome.records,
        semantic_outcome.records,
        errors,
    )
}
