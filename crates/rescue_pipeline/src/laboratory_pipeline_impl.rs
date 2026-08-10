use crate::laboratory_pipeline_read::ReadStage;
use crate::laboratory_pipeline_resolve::{ResolveFailure, ResolveStage};
use crate::laboratory_pipeline_write::{WriteFailure, WriteProgress, WriteStage};
use crate::{LaboratoryPackageRequest, LaboratoryPackageResult, LABORATORY_PIPELINE_VERSION};

pub(crate) fn run_laboratory_package_impl(
    request: &LaboratoryPackageRequest,
) -> LaboratoryPackageResult {
    let mut result = empty_result(request);
    let input_errors = crate::laboratory_pipeline_inputs::validate_request(request);
    if !input_errors.is_empty() {
        result.run_status = "rejected_before_read".to_string();
        result.completed_stage = "request_validation".to_string();
        result.errors = input_errors;
        return result;
    }
    let read = match crate::laboratory_pipeline_read::read_and_assess(request) {
        Ok(read) => read,
        Err(failure) => {
            result.run_status = "read_stage_failed".to_string();
            result.completed_stage = failure.error.stage.clone();
            result.discovery = Some(failure.discovery);
            result.errors.push(failure.error);
            return result;
        }
    };
    let resolved = match crate::laboratory_pipeline_resolve::resolve_and_plan(request, &read) {
        Ok(resolved) => resolved,
        Err(failure) => {
            store_read(&mut result, read);
            store_resolve_failure(&mut result, *failure);
            result.run_status = "resolution_or_plan_blocked".to_string();
            return result;
        }
    };
    let written =
        match crate::laboratory_pipeline_write::execute_package(request, &resolved.package_plan) {
            Ok(written) => written,
            Err(failure) => {
                store_read(&mut result, read);
                store_resolve(&mut result, resolved);
                store_write_failure(&mut result, *failure);
                result.run_status = "write_pipeline_failed".to_string();
                return result;
            }
        };
    store_read(&mut result, read);
    store_resolve(&mut result, resolved);
    store_write(&mut result, written);
    result.run_status = "ready_for_manual_ableton_check".to_string();
    result.completed_stage = "promotion".to_string();
    result
}

fn empty_result(request: &LaboratoryPackageRequest) -> LaboratoryPackageResult {
    LaboratoryPackageResult {
        pipeline_version: LABORATORY_PIPELINE_VERSION.to_string(),
        run_id: request.run_id.clone(),
        run_status: "not_started".to_string(),
        completed_stage: "none".to_string(),
        discovery: None,
        als_read_model: None,
        extraction: None,
        path_observations: None,
        assessment: None,
        preflight: None,
        inventory: None,
        resolution: None,
        package_plan: None,
        staging: None,
        rewrite: None,
        validation: None,
        manifests: None,
        promotion: None,
        errors: Vec::new(),
    }
}

fn store_read(result: &mut LaboratoryPackageResult, read: ReadStage) {
    result.discovery = Some(read.discovery);
    result.als_read_model = Some(read.als_read_model);
    result.extraction = Some(read.extraction);
    result.path_observations = Some(read.path_observations);
    result.assessment = Some(read.assessment);
    result.preflight = Some(read.preflight);
    result.completed_stage = "preflight".to_string();
}

fn store_resolve(result: &mut LaboratoryPackageResult, resolved: ResolveStage) {
    result.inventory = Some(resolved.inventory);
    result.resolution = Some(resolved.resolution);
    result.package_plan = Some(resolved.package_plan);
    result.completed_stage = "package_planning".to_string();
}

fn store_resolve_failure(result: &mut LaboratoryPackageResult, failure: ResolveFailure) {
    result.inventory = failure.inventory;
    result.resolution = failure.resolution;
    result.package_plan = failure.package_plan;
    result.completed_stage = failure.error.stage.clone();
    result.errors.push(failure.error);
}

fn store_write(result: &mut LaboratoryPackageResult, written: WriteStage) {
    result.staging = Some(written.staging);
    result.rewrite = Some(written.rewrite);
    result.validation = Some(written.validation);
    result.manifests = Some(written.manifests);
    result.promotion = Some(written.promotion);
}

fn store_write_failure(result: &mut LaboratoryPackageResult, failure: WriteFailure) {
    store_progress(result, failure.progress);
    result.completed_stage = failure.error.stage.clone();
    result.errors.push(failure.error);
}

fn store_progress(result: &mut LaboratoryPackageResult, progress: WriteProgress) {
    result.staging = progress.staging;
    result.rewrite = progress.rewrite;
    result.validation = progress.validation;
    result.manifests = progress.manifests;
    result.promotion = progress.promotion;
}
