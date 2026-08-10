use crate::laboratory_pipeline_read::ReadStage;
use crate::{LaboratoryPackageError, LaboratoryPackageRequest};
use rescue_catalog::{scan_assets, AssetInventoryRequest, AssetInventoryResult};
use rescue_packaging::{plan_package, PackagePlan, PackagePlanningRequest};
use rescue_resolution::{resolve_assets_with_selections, AssetResolutionResult};

pub(crate) struct ResolveStage {
    pub inventory: AssetInventoryResult,
    pub resolution: AssetResolutionResult,
    pub package_plan: PackagePlan,
}

pub(crate) struct ResolveFailure {
    pub error: LaboratoryPackageError,
    pub inventory: Option<AssetInventoryResult>,
    pub resolution: Option<AssetResolutionResult>,
    pub package_plan: Option<PackagePlan>,
}

pub(crate) fn resolve_and_plan(
    request: &LaboratoryPackageRequest,
    read: &ReadStage,
) -> Result<ResolveStage, Box<ResolveFailure>> {
    let inventory = scan_assets(&AssetInventoryRequest {
        scan_run_id: format!("{}:inventory", request.run_id),
        roots: request.scan_roots.clone(),
        max_entries: request.max_scan_entries,
    });
    if !inventory.errors.is_empty() {
        return Err(failure(
            "PIPELINE_ASSET_INVENTORY_FAILED",
            "asset_inventory",
            "Selected-scope asset inventory returned errors",
            Some(inventory),
            None,
            None,
        ));
    }
    let resolution = resolve_assets_with_selections(
        &read.assessment,
        &inventory,
        request.user_selection_set.as_ref(),
    );
    if !resolution.errors.is_empty() {
        return Err(failure(
            "PIPELINE_ASSET_RESOLUTION_FAILED",
            "asset_resolution",
            "Asset resolution returned errors",
            Some(inventory),
            Some(resolution),
            None,
        ));
    }
    let package_plan = plan_package(
        &PackagePlanningRequest {
            plan_id: format!("{}:plan", request.run_id),
            target_project_root: request.target_project_root.clone(),
            planning_mode: "laboratory_rescue_rewrite".to_string(),
        },
        &read.als_read_model,
        &read.assessment,
        &inventory,
        &resolution,
    );
    if package_plan.plan_status != "ready_for_laboratory_execution" {
        return Err(failure(
            "PIPELINE_PACKAGE_PLAN_BLOCKED",
            "package_planning",
            "Package plan has unresolved requirements or safety errors",
            Some(inventory),
            Some(resolution),
            Some(package_plan),
        ));
    }
    Ok(ResolveStage {
        inventory,
        resolution,
        package_plan,
    })
}

fn failure(
    code: &str,
    stage: &str,
    message: &str,
    inventory: Option<AssetInventoryResult>,
    resolution: Option<AssetResolutionResult>,
    package_plan: Option<PackagePlan>,
) -> Box<ResolveFailure> {
    Box::new(ResolveFailure {
        error: LaboratoryPackageError {
            error_code: code.to_string(),
            stage: stage.to_string(),
            message: message.to_string(),
        },
        inventory,
        resolution,
        package_plan,
    })
}
