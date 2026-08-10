use crate::{
    ProjectCatalogApplicationError, ProjectCatalogApplicationWarning, ProjectSelection,
    ProjectSelectionRequest, ProjectSelectionResult, PROJECT_CATALOG_APPLICATION_SERVICE_VERSION,
};
use rescue_catalog::{load_project_catalog, StoredProjectCatalog};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;

pub(crate) fn resolve_project_selection_impl(
    request: &ProjectSelectionRequest,
) -> ProjectSelectionResult {
    if request.request_id.trim().is_empty()
        || (request.catalog_live_set_ids.is_empty() && request.manual_als_paths.is_empty())
    {
        return failure(
            request,
            "PROJECT_SELECTION_EMPTY",
            "selection",
            "At least one explicit Project selection is required.",
        );
    }
    if has_duplicates(&request.catalog_live_set_ids) {
        return failure(
            request,
            "PROJECT_SELECTION_DUPLICATE_ID",
            "selection",
            "A catalog Live Set ID was selected more than once.",
        );
    }
    let mut warnings = Vec::new();
    let mut selections = match catalog_selections(request, &mut warnings) {
        Ok(selections) => selections,
        Err(error) => return failure_from_error(request, error),
    };
    for path in &request.manual_als_paths {
        let selection = match manual_selection(path) {
            Ok(selection) => selection,
            Err(error) => return failure_from_error(request, error),
        };
        selections.push(selection);
    }
    if has_duplicate_paths(&selections) {
        return failure(
            request,
            "PROJECT_SELECTION_DUPLICATE_PATH",
            "selection",
            "Two selections resolve to the same native ALS path.",
        );
    }
    ProjectSelectionResult {
        service_version: PROJECT_CATALOG_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        selection_status: "resolved".to_string(),
        selections,
        warnings,
        errors: Vec::new(),
    }
}

fn catalog_selections(
    request: &ProjectSelectionRequest,
    warnings: &mut Vec<ProjectCatalogApplicationWarning>,
) -> Result<Vec<ProjectSelection>, ProjectCatalogApplicationError> {
    if request.catalog_live_set_ids.is_empty() {
        return Ok(Vec::new());
    }
    let revision = request.catalog_revision.ok_or_else(|| {
        error(
            "PROJECT_SELECTION_REVISION_REQUIRED",
            "selection",
            "Catalog revision is required for catalog selections.",
        )
    })?;
    let loaded = load_project_catalog(&request.store_path);
    let catalog = loaded.catalog.as_ref().ok_or_else(|| {
        let message = loaded
            .errors
            .first()
            .map(|error| error.message.as_str())
            .unwrap_or("The private Project catalog is unavailable.");
        error("PROJECT_CATALOG_STORE_FAILED", "store", message)
    })?;
    if catalog.metadata.revision != revision {
        return Err(error(
            "PROJECT_SELECTION_STALE_REVISION",
            "selection",
            "The Project catalog changed after the list was displayed.",
        ));
    }
    request
        .catalog_live_set_ids
        .iter()
        .map(|id| catalog_selection(id, catalog, revision, warnings))
        .collect()
}

fn catalog_selection(
    id: &str,
    catalog: &StoredProjectCatalog,
    revision: u64,
    warnings: &mut Vec<ProjectCatalogApplicationWarning>,
) -> Result<ProjectSelection, ProjectCatalogApplicationError> {
    let stored = catalog
        .live_sets
        .iter()
        .find(|stored| stored.record.live_set_id == id)
        .ok_or_else(|| {
            error(
                "PROJECT_SELECTION_UNKNOWN_ID",
                "selection",
                "A selected catalog Live Set ID does not exist.",
            )
        })?;
    if stored.freshness_status == "not_observed_in_latest_complete_scan" {
        return Err(error(
            "PROJECT_SELECTION_STALE_ITEM",
            "selection",
            "A selected Live Set was not observed by the latest complete scan.",
        ));
    }
    if stored.freshness_status == "retained_from_prior_scan" {
        warnings.push(ProjectCatalogApplicationWarning {
            warning_code: "PROJECT_SELECTION_PARTIAL_COVERAGE".to_string(),
            message: "A selected Live Set was retained from a prior scan and requires downstream verification."
                .to_string(),
        });
    }
    Ok(ProjectSelection {
        selection_id: selection_id(
            "catalog",
            &stored.record.native_als_path,
            Some(&stored.record.observation_fingerprint),
        )?,
        selection_source: "catalog".to_string(),
        live_set_id: Some(stored.record.live_set_id.clone()),
        native_als_path: stored.record.native_als_path.clone(),
        catalog_revision: Some(revision),
        observation_fingerprint: Some(stored.record.observation_fingerprint.clone()),
        freshness_status: stored.freshness_status.clone(),
    })
}

fn manual_selection(path: &Path) -> Result<ProjectSelection, ProjectCatalogApplicationError> {
    if !path.is_absolute()
        || !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("als"))
        || path.to_str().is_none()
    {
        return Err(error(
            "PROJECT_SELECTION_MANUAL_PATH_INVALID",
            "selection",
            "Manual selection requires one exact absolute ALS path.",
        ));
    }
    Ok(ProjectSelection {
        selection_id: selection_id("manual", path, None)?,
        selection_source: "manual".to_string(),
        live_set_id: None,
        native_als_path: path.to_path_buf(),
        catalog_revision: None,
        observation_fingerprint: None,
        freshness_status: "manual_unverified".to_string(),
    })
}

fn selection_id(
    source: &str,
    path: &Path,
    fingerprint: Option<&str>,
) -> Result<String, ProjectCatalogApplicationError> {
    let path = path.to_str().ok_or_else(|| {
        error(
            "PROJECT_SELECTION_MANUAL_PATH_INVALID",
            "selection",
            "Selection path cannot be represented exactly.",
        )
    })?;
    let mut hasher = Sha256::new();
    for value in [source, path, fingerprint.unwrap_or("")] {
        hasher.update((value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn has_duplicates(values: &[String]) -> bool {
    let mut seen = HashSet::new();
    values.iter().any(|value| !seen.insert(value))
}

fn has_duplicate_paths(selections: &[ProjectSelection]) -> bool {
    let mut seen = HashSet::new();
    selections
        .iter()
        .filter_map(|selection| path_key(&selection.native_als_path))
        .any(|path| !seen.insert(path))
}

fn path_key(path: &Path) -> Option<String> {
    let value = path.to_str()?;
    if cfg!(windows) {
        Some(value.to_lowercase())
    } else {
        Some(value.to_string())
    }
}

fn failure_from_error(
    request: &ProjectSelectionRequest,
    application_error: ProjectCatalogApplicationError,
) -> ProjectSelectionResult {
    ProjectSelectionResult {
        service_version: PROJECT_CATALOG_APPLICATION_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        selection_status: "failed".to_string(),
        selections: Vec::new(),
        warnings: Vec::new(),
        errors: vec![application_error],
    }
}

fn failure(
    request: &ProjectSelectionRequest,
    code: &str,
    stage: &str,
    message: &str,
) -> ProjectSelectionResult {
    failure_from_error(request, error(code, stage, message))
}

fn error(code: &str, stage: &str, message: &str) -> ProjectCatalogApplicationError {
    ProjectCatalogApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
