use crate::{
    project_catalog_projection, ProjectCatalogApplicationError, ProjectCatalogApplicationWarning,
    ProjectCatalogRefreshRequest, ProjectCatalogRefreshResult,
    PROJECT_CATALOG_APPLICATION_SERVICE_VERSION,
};
use rescue_catalog::{
    build_project_catalog, scan_projects, store_project_catalog, ProjectCatalogBuildRequest,
    ProjectCatalogStoreRequest, ProjectScanRequest, PROJECT_SCAN_TRAVERSAL_POLICY,
};
use sha2::{Digest, Sha256};

pub(crate) fn refresh_project_catalog_impl(
    request: &ProjectCatalogRefreshRequest,
) -> ProjectCatalogRefreshResult {
    if request.request_id.trim().is_empty() {
        return failure(
            request,
            "request",
            "Project catalog request ID must not be empty.",
        );
    }
    let coverage_scope_id = match coverage_scope_id(request) {
        Ok(value) => value,
        Err(message) => return failure(request, "request", message),
    };
    let scan = scan_projects(&ProjectScanRequest {
        scan_run_id: format!("{}:scan", request.request_id),
        roots: request.roots.clone(),
        excluded_roots: request.excluded_roots.clone(),
        max_entries: request.max_entries,
        max_depth: request.max_depth,
        traversal_policy_version: PROJECT_SCAN_TRAVERSAL_POLICY.to_string(),
    });
    if scan.metadata.scan_status == "failed" {
        return scan_failure(request, &scan);
    }
    let scan_status = scan.metadata.scan_status.clone();
    let scan_warnings = scan
        .warnings
        .iter()
        .map(|warning| ProjectCatalogApplicationWarning {
            warning_code: warning.warning_code.clone(),
            message: warning.message.clone(),
        })
        .collect::<Vec<_>>();
    let snapshot = build_project_catalog(ProjectCatalogBuildRequest {
        snapshot_id: format!("{}:snapshot", request.request_id),
        scan_result: scan,
    });
    if snapshot.metadata.build_status == "failed" {
        return catalog_failure(request, scan_status, &snapshot.errors);
    }
    let stored = store_project_catalog(&ProjectCatalogStoreRequest {
        store_path: request.store_path.clone(),
        coverage_scope_id,
        snapshot,
    });
    if !stored.errors.is_empty() {
        return store_failure(request, scan_status, &stored);
    }
    let Some(catalog) = stored.catalog.as_ref() else {
        return failure(request, "store", "Project catalog store returned no state.");
    };
    let list = project_catalog_projection::project_stored_catalog(
        catalog,
        request.include_backups,
        request.include_stale,
    );
    let refresh_status = match scan_status.as_str() {
        "partial" => "partial",
        "cancelled" => "cancelled",
        _ => "complete",
    };
    ProjectCatalogRefreshResult {
        service_version: PROJECT_CATALOG_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        refresh_status: refresh_status.to_string(),
        scan_status,
        store_status: stored.operation_status,
        catalog: list,
        warnings: scan_warnings,
        errors: Vec::new(),
    }
}

fn coverage_scope_id(request: &ProjectCatalogRefreshRequest) -> Result<String, &'static str> {
    let mut roots = exact_sorted_paths(&request.roots)?;
    let mut exclusions = exact_sorted_paths(&request.excluded_roots)?;
    roots.sort();
    roots.dedup();
    exclusions.sort();
    exclusions.dedup();
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, PROJECT_SCAN_TRAVERSAL_POLICY);
    hash_field(
        &mut hasher,
        &request
            .max_depth
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    for root in roots {
        hash_field(&mut hasher, "root");
        hash_field(&mut hasher, root);
    }
    for exclusion in exclusions {
        hash_field(&mut hasher, "exclude");
        hash_field(&mut hasher, exclusion);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn exact_sorted_paths(paths: &[std::path::PathBuf]) -> Result<Vec<&str>, &'static str> {
    paths
        .iter()
        .map(|path| {
            path.to_str()
                .ok_or("Project scan roots must be represented exactly.")
        })
        .collect()
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn scan_failure(
    request: &ProjectCatalogRefreshRequest,
    scan: &rescue_catalog::ProjectScanResult,
) -> ProjectCatalogRefreshResult {
    let message = scan
        .errors
        .first()
        .map(|error| error.message.as_str())
        .unwrap_or("Project scan failed.");
    failure(request, "scan", message)
}

fn catalog_failure(
    request: &ProjectCatalogRefreshRequest,
    scan_status: String,
    errors: &[rescue_catalog::ProjectCatalogError],
) -> ProjectCatalogRefreshResult {
    let message = errors
        .first()
        .map(|error| error.message.as_str())
        .unwrap_or("Project catalog build failed.");
    failure_with_scan(request, "catalog", message, scan_status)
}

fn store_failure(
    request: &ProjectCatalogRefreshRequest,
    scan_status: String,
    stored: &rescue_catalog::ProjectCatalogStoreResult,
) -> ProjectCatalogRefreshResult {
    let message = stored
        .errors
        .first()
        .map(|error| error.message.as_str())
        .unwrap_or("Project catalog persistence failed.");
    failure_with_scan(request, "store", message, scan_status)
}

fn failure(
    request: &ProjectCatalogRefreshRequest,
    stage: &str,
    message: &str,
) -> ProjectCatalogRefreshResult {
    failure_with_scan(request, stage, message, "failed".to_string())
}

fn failure_with_scan(
    request: &ProjectCatalogRefreshRequest,
    stage: &str,
    message: &str,
    scan_status: String,
) -> ProjectCatalogRefreshResult {
    ProjectCatalogRefreshResult {
        service_version: PROJECT_CATALOG_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        refresh_status: "failed".to_string(),
        scan_status,
        store_status: "failed".to_string(),
        catalog: project_catalog_projection::empty_list(),
        warnings: Vec::new(),
        errors: vec![ProjectCatalogApplicationError {
            error_code: "PROJECT_CATALOG_REFRESH_FAILED".to_string(),
            stage: stage.to_string(),
            message: message.to_string(),
        }],
    }
}
