use crate::{ALSRewriteRequest, ALSRewriteResult};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;

pub(crate) fn rewrite_staged_als_impl(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
) -> ALSRewriteResult {
    let errors = crate::als_rewriter_validation::validate_inputs(request, plan, staging);
    if !errors.is_empty() {
        return output(
            request,
            plan,
            staging,
            None,
            None,
            Vec::new(),
            "rejected",
            errors,
        );
    }
    let staged_als = request.staging_root.join(&staging.staged_als_relative_path);
    let decoded = match crate::als_rewriter_io::read_staged_als(&staged_als) {
        Ok(decoded) => decoded,
        Err(failure) => {
            return failure_output(request, plan, staging, None, failure, Some(staged_als))
        }
    };
    if decoded.compressed_hash != plan.source_als.source_file_hash {
        let error = crate::als_rewriter_result::rewrite_error(
            "REWRITE_SOURCE_HASH_MISMATCH",
            "Staged ALS no longer matches the planned source snapshot".to_string(),
            None,
            Some(staged_als),
        );
        return output(
            request,
            plan,
            staging,
            Some(decoded.compressed_hash),
            None,
            Vec::new(),
            "rewrite_failed",
            vec![error],
        );
    }
    if plan.rewrite_operations.is_empty() {
        return output(
            request,
            plan,
            staging,
            Some(decoded.compressed_hash.clone()),
            Some(decoded.compressed_hash),
            Vec::new(),
            "not_required",
            Vec::new(),
        );
    }
    let rewritten =
        match crate::als_rewriter_xml::rewrite_xml(&decoded.xml, &plan.rewrite_operations) {
            Ok(outcome) => outcome,
            Err(failure) => {
                let error = crate::als_rewriter_result::rewrite_error(
                    failure.code,
                    failure.message,
                    failure.operation_id,
                    Some(staged_als),
                );
                return output(
                    request,
                    plan,
                    staging,
                    Some(decoded.compressed_hash),
                    None,
                    Vec::new(),
                    "rewrite_failed",
                    vec![error],
                );
            }
        };
    complete_rewrite(
        request,
        plan,
        staging,
        staged_als,
        decoded.compressed_hash,
        rewritten,
    )
}

fn complete_rewrite(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    staged_als: std::path::PathBuf,
    original_hash: String,
    rewritten: crate::als_rewriter_xml::XmlRewriteOutcome,
) -> ALSRewriteResult {
    let compressed = match crate::als_rewriter_io::encode_xml(&rewritten.xml) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return failure_output(request, plan, staging, Some(original_hash), failure, None)
        }
    };
    if let Err(failure) = crate::als_rewriter_io::validate_encoded_als(&compressed, &staged_als) {
        return failure_output(
            request,
            plan,
            staging,
            Some(original_hash),
            failure,
            Some(staged_als),
        );
    }
    let rewritten_hash = crate::als_rewriter_io::sha256_hex(&compressed);
    if let Err(failure) = crate::als_rewriter_io::replace_staged_als(&staged_als, &compressed) {
        return failure_output(
            request,
            plan,
            staging,
            Some(original_hash),
            failure,
            Some(staged_als),
        );
    }
    let records = crate::als_rewriter_result::records_for(
        &plan.rewrite_operations,
        &rewritten.completed_operation_ids,
    );
    output(
        request,
        plan,
        staging,
        Some(original_hash),
        Some(rewritten_hash),
        records,
        "rewrite_complete",
        Vec::new(),
    )
}

fn failure_output(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    original_hash: Option<String>,
    failure: crate::als_rewriter_io::RewriteIoFailure,
    fallback_path: Option<std::path::PathBuf>,
) -> ALSRewriteResult {
    let error = crate::als_rewriter_result::rewrite_error(
        failure.code,
        failure.message,
        None,
        failure.path.or(fallback_path),
    );
    output(
        request,
        plan,
        staging,
        original_hash,
        None,
        Vec::new(),
        "rewrite_failed",
        vec![error],
    )
}

#[allow(clippy::too_many_arguments)]
fn output(
    request: &ALSRewriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    original_hash: Option<String>,
    rewritten_hash: Option<String>,
    records: Vec<crate::RewriteExecutionRecord>,
    status: &str,
    errors: Vec<crate::ALSRewriteError>,
) -> ALSRewriteResult {
    crate::als_rewriter_result::result(
        request,
        plan,
        staging,
        original_hash,
        rewritten_hash,
        records,
        status,
        errors,
    )
}
