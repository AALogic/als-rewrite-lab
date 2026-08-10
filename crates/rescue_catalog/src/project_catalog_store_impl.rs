use crate::{
    project_catalog_store_io, project_catalog_store_merge, project_catalog_store_validation,
    ProjectCatalogLoadResult, ProjectCatalogStoreError, ProjectCatalogStoreRequest,
    ProjectCatalogStoreResult,
};
use std::path::Path;

pub(crate) fn store_project_catalog_impl(
    request: &ProjectCatalogStoreRequest,
) -> ProjectCatalogStoreResult {
    if let Err(error) = project_catalog_store_validation::validate_store_request(request) {
        return store_failure(error);
    }
    let prior = match project_catalog_store_io::read_catalog(&request.store_path) {
        Ok(catalog) => catalog,
        Err(error) => return store_failure(error),
    };
    let (candidate, warnings) = project_catalog_store_merge::merge_snapshot(
        prior.as_ref(),
        &request.coverage_scope_id,
        &request.snapshot,
    );
    if let Err(error) = project_catalog_store_validation::validate_stored_catalog(&candidate) {
        return store_failure(error);
    }
    if prior.as_ref().is_some_and(|current| {
        project_catalog_store_merge::semantically_unchanged(current, &candidate)
    }) {
        return ProjectCatalogStoreResult {
            operation_status: "already_current".to_string(),
            catalog: prior,
            warnings,
            errors: Vec::new(),
        };
    }
    if let Err(error) = project_catalog_store_io::write_catalog(&request.store_path, &candidate) {
        return store_failure(error);
    }
    ProjectCatalogStoreResult {
        operation_status: "stored".to_string(),
        catalog: Some(candidate),
        warnings,
        errors: Vec::new(),
    }
}

pub(crate) fn load_project_catalog_impl(store_path: &Path) -> ProjectCatalogLoadResult {
    if !store_path.is_absolute() || store_path.file_name().is_none() {
        return load_failure(project_catalog_store_validation::store_error(
            "CATALOG_STORE_PATH_INVALID",
            "The private catalog store path must be an absolute file path.",
        ));
    }
    match project_catalog_store_io::read_catalog(store_path) {
        Ok(Some(catalog)) => ProjectCatalogLoadResult {
            load_status: "loaded".to_string(),
            catalog: Some(catalog),
            errors: Vec::new(),
        },
        Ok(None) => ProjectCatalogLoadResult {
            load_status: "not_found".to_string(),
            catalog: None,
            errors: Vec::new(),
        },
        Err(error) => load_failure(error),
    }
}

fn store_failure(error: ProjectCatalogStoreError) -> ProjectCatalogStoreResult {
    ProjectCatalogStoreResult {
        operation_status: "failed".to_string(),
        catalog: None,
        warnings: Vec::new(),
        errors: vec![error],
    }
}

fn load_failure(error: ProjectCatalogStoreError) -> ProjectCatalogLoadResult {
    ProjectCatalogLoadResult {
        load_status: "failed".to_string(),
        catalog: None,
        errors: vec![error],
    }
}
