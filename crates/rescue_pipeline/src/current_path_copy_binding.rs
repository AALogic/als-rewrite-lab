use crate::current_path_copy::{CurrentPathCopyRequest, CurrentPathCopyResult};
use crate::current_path_copy_impl::{error, failure_result, laboratory_request};
use crate::laboratory_pipeline_read::ReadStage;
use crate::LaboratoryPackageError;
use rescue_packaging::{
    fingerprint_package_plan, plan_current_path_package, PackagePlan, PackagePlanningRequest,
    PlanFingerprint, COMPATIBILITY_LAB_CURRENT_PATHS_MODE, STRICT_CURRENT_PATHS_MODE,
};
use rescue_resolution::bind_current_paths;

pub(crate) struct PreparedCopy {
    pub(crate) plan: PackagePlan,
    pub(crate) fingerprint: PlanFingerprint,
}

pub(crate) fn prepare_stage(
    request: &CurrentPathCopyRequest,
) -> Result<PreparedCopy, Box<CurrentPathCopyResult>> {
    let planning_mode = match request.rewrite_policy.as_str() {
        crate::STRICT_REWRITE_POLICY => STRICT_CURRENT_PATHS_MODE,
        crate::COMPATIBILITY_LAB_REWRITE_POLICY => COMPATIBILITY_LAB_CURRENT_PATHS_MODE,
        _ => {
            return Err(Box::new(failure_result(
                request,
                "rejected_before_read",
                "request_validation",
                None,
                None,
                vec![error(
                    "CURRENT_PATH_REWRITE_POLICY_UNSUPPORTED",
                    "request_validation",
                    "Current-path rewrite policy is unsupported",
                )],
            )))
        }
    };
    let lab_request = laboratory_request(request);
    let input_errors = crate::laboratory_pipeline_inputs::validate_request(&lab_request);
    if !input_errors.is_empty() {
        return Err(Box::new(failure_result(
            request,
            "rejected_before_read",
            "request_validation",
            None,
            None,
            input_errors,
        )));
    }
    let read =
        crate::laboratory_pipeline_read::read_and_assess(&lab_request).map_err(|failure| {
            let stage = failure.error.stage.clone();
            Box::new(failure_result(
                request,
                "read_stage_failed",
                &stage,
                None,
                None,
                vec![failure.error],
            ))
        })?;
    if request
        .expected_source_als_sha256
        .as_deref()
        .is_some_and(|expected| expected != read.als_read_model.set_metadata.source_file_hash)
    {
        return Err(Box::new(failure_result(
            request,
            "snapshot_or_plan_blocked",
            "source_snapshot",
            None,
            None,
            vec![error(
                "CURRENT_PATH_SOURCE_SNAPSHOT_CHANGED",
                "source_snapshot",
                "Source ALS no longer matches the accepted desktop preview",
            )],
        )));
    }
    plan_current_paths(request, read, planning_mode)
}

fn plan_current_paths(
    request: &CurrentPathCopyRequest,
    read: ReadStage,
    planning_mode: &str,
) -> Result<PreparedCopy, Box<CurrentPathCopyResult>> {
    let bindings = bind_current_paths(&read.assessment);
    if !bindings.errors.is_empty() {
        return Err(Box::new(failure_result(
            request,
            "snapshot_or_plan_blocked",
            "current_path_binding",
            None,
            None,
            vec![error(
                "CURRENT_PATH_BINDING_FAILED",
                "current_path_binding",
                "Current recorded paths could not be bound safely",
            )],
        )));
    }
    let plan = plan_current_path_package(
        &PackagePlanningRequest {
            plan_id: format!("{}:plan", request.run_id),
            target_project_root: request.target_project_root.clone(),
            planning_mode: planning_mode.to_string(),
        },
        &read.als_read_model,
        &read.assessment,
        &bindings,
    );
    if !matches!(
        plan.plan_status.as_str(),
        "ready_current_paths_complete" | "ready_current_paths_incomplete"
    ) {
        let blockers = plan_blockers(&plan);
        return Err(Box::new(failure_result(
            request,
            "snapshot_or_plan_blocked",
            "package_planning",
            Some(plan),
            None,
            blockers,
        )));
    }
    let fingerprint = fingerprint_package_plan(&plan).map_err(|failure| {
        Box::new(failure_result(
            request,
            "snapshot_or_plan_blocked",
            "plan_fingerprint",
            Some(plan.clone()),
            None,
            vec![error(
                "CURRENT_PATH_PLAN_FINGERPRINT_FAILED",
                "plan_fingerprint",
                &format!("Package plan could not be fingerprinted: {failure}"),
            )],
        ))
    })?;
    Ok(PreparedCopy { plan, fingerprint })
}

fn plan_blockers(plan: &PackagePlan) -> Vec<LaboratoryPackageError> {
    let mut blockers: Vec<_> = plan
        .errors
        .iter()
        .map(|item| {
            let path = item
                .path
                .as_ref()
                .map(|value| format!(" ({})", value.display()))
                .unwrap_or_default();
            error(
                &item.error_code,
                "package_planning",
                &format!("{}{path}", item.message),
            )
        })
        .collect();
    blockers.extend(
        plan.unresolved_requirements
            .iter()
            .filter(|item| item.blocks_execution)
            .map(|item| {
                error(
                    "CURRENT_PATH_UNRESOLVED_BLOCKER",
                    "package_planning",
                    &format!(
                        "Required asset {} is blocked: {}",
                        item.required_asset_id, item.reason
                    ),
                )
            }),
    );
    if blockers.is_empty() {
        blockers.push(error(
            "CURRENT_PATH_PACKAGE_PLAN_BLOCKED",
            "package_planning",
            "Current-path copy plan contains an unspecified safety blocker",
        ));
    }
    blockers
}
