use crate::{
    ProjectCatalogApplicationError, ProjectCatalogApplicationWarning, ProjectCatalogListMetadata,
    ProjectCatalogListRequest, ProjectCatalogListResult, ProjectListItem,
    PROJECT_CATALOG_APPLICATION_SERVICE_VERSION,
};
use rescue_catalog::{load_project_catalog, StoredLiveSetRecord, StoredProjectCatalog};
use std::collections::{HashMap, HashSet};

pub(crate) fn list_project_catalog_impl(
    request: &ProjectCatalogListRequest,
) -> ProjectCatalogListResult {
    let loaded = load_project_catalog(&request.store_path);
    let Some(catalog) = loaded.catalog.as_ref() else {
        let message = loaded
            .errors
            .first()
            .map(|error| error.message.as_str())
            .unwrap_or("No private Project catalog has been created yet.");
        let code = if loaded.load_status == "not_found" {
            "PROJECT_CATALOG_NOT_FOUND"
        } else {
            "PROJECT_CATALOG_STORE_FAILED"
        };
        return failed_list(code, "store", message);
    };
    project_stored_catalog(catalog, request.include_backups, request.include_stale)
}

pub(crate) fn project_stored_catalog(
    catalog: &StoredProjectCatalog,
    include_backups: bool,
    include_stale: bool,
) -> ProjectCatalogListResult {
    let folder_names = catalog
        .project_folders
        .iter()
        .map(|stored| {
            (
                stored.record.project_folder_id.as_str(),
                stored.record.display_name.as_str(),
            )
        })
        .collect::<HashMap<_, _>>();
    let hidden_backup_count = catalog
        .live_sets
        .iter()
        .filter(|stored| is_backup(stored) && !include_backups)
        .count();
    let hidden_stale_count = catalog
        .live_sets
        .iter()
        .filter(|stored| is_stale(stored) && !include_stale)
        .count();
    let total_group_count = catalog
        .live_sets
        .iter()
        .map(crate::project_catalog_groups::group_id)
        .collect::<HashSet<_>>()
        .len();
    let mut items = catalog
        .live_sets
        .iter()
        .filter(|stored| include_backups || !is_backup(stored))
        .filter(|stored| include_stale || !is_stale(stored))
        .map(|stored| list_item(stored, &folder_names))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        display_sort_key(left)
            .cmp(&display_sort_key(right))
            .then(left.live_set_id.cmp(&right.live_set_id))
    });
    apply_duplicate_counts(&mut items);
    let groups = crate::project_catalog_groups::build_groups(&items, catalog);
    let warnings = coverage_warnings(catalog);
    ProjectCatalogListResult {
        metadata: ProjectCatalogListMetadata {
            service_version: PROJECT_CATALOG_APPLICATION_SERVICE_VERSION.to_string(),
            catalog_revision: catalog.metadata.revision,
            coverage_status: catalog.metadata.coverage_status.clone(),
            total_group_count,
            total_item_count: catalog.live_sets.len(),
            visible_item_count: items.len(),
            hidden_backup_count,
            hidden_stale_count,
        },
        groups,
        items,
        warnings,
        errors: Vec::new(),
    }
}

pub(crate) fn empty_list() -> ProjectCatalogListResult {
    ProjectCatalogListResult {
        metadata: ProjectCatalogListMetadata {
            service_version: PROJECT_CATALOG_APPLICATION_SERVICE_VERSION.to_string(),
            catalog_revision: 0,
            coverage_status: "unavailable".to_string(),
            total_group_count: 0,
            total_item_count: 0,
            visible_item_count: 0,
            hidden_backup_count: 0,
            hidden_stale_count: 0,
        },
        groups: Vec::new(),
        items: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn list_item(stored: &StoredLiveSetRecord, folder_names: &HashMap<&str, &str>) -> ProjectListItem {
    let warning_codes = item_warnings(stored);
    let selection_status = if is_stale(stored) {
        "stale_unavailable"
    } else if warning_codes.is_empty() {
        "selectable"
    } else {
        "selectable_with_warning"
    };
    ProjectListItem {
        item_id: stored.record.live_set_id.clone(),
        live_set_id: stored.record.live_set_id.clone(),
        group_id: crate::project_catalog_groups::group_id(stored),
        display_name: stored.record.display_name.clone(),
        project_display_name: stored
            .record
            .project_folder_id
            .as_deref()
            .and_then(|id| folder_names.get(id).copied())
            .map(str::to_string),
        location_kind: stored.record.location_kind.clone(),
        freshness_status: stored.freshness_status.clone(),
        selection_status: selection_status.to_string(),
        modified_time_unix_ms: stored.record.modified_time_unix_ms,
        file_size: stored.record.file_size,
        duplicate_display_name_count: 1,
        warning_codes,
    }
}

fn item_warnings(stored: &StoredLiveSetRecord) -> Vec<String> {
    let mut warnings = Vec::new();
    if stored.freshness_status == "retained_from_prior_scan" {
        warnings.push("PROJECT_CATALOG_PARTIAL_COVERAGE".to_string());
    }
    if stored.record.association_status == "ambiguous_project_context" {
        warnings.push("PROJECT_CATALOG_AMBIGUOUS_CONTEXT".to_string());
    } else if stored.record.association_status == "ungrouped" {
        warnings.push("PROJECT_CATALOG_UNGROUPED_SET".to_string());
    }
    if is_backup(stored) {
        warnings.push("PROJECT_CATALOG_BACKUP_SET".to_string());
    }
    if is_stale(stored) {
        warnings.push("PROJECT_CATALOG_STALE_SET".to_string());
    }
    warnings
}

fn apply_duplicate_counts(items: &mut [ProjectListItem]) {
    let counts = items.iter().fold(HashMap::new(), |mut counts, item| {
        *counts.entry(item.display_name.clone()).or_insert(0usize) += 1;
        counts
    });
    for item in items {
        item.duplicate_display_name_count = counts.get(&item.display_name).copied().unwrap_or(1);
    }
}

fn display_sort_key(item: &ProjectListItem) -> (String, String) {
    (
        item.project_display_name
            .as_deref()
            .unwrap_or("")
            .to_lowercase(),
        item.display_name.to_lowercase(),
    )
}

fn is_backup(stored: &StoredLiveSetRecord) -> bool {
    stored.record.location_kind.starts_with("backup")
}

fn is_stale(stored: &StoredLiveSetRecord) -> bool {
    stored.freshness_status == "not_observed_in_latest_complete_scan"
}

fn coverage_warnings(catalog: &StoredProjectCatalog) -> Vec<ProjectCatalogApplicationWarning> {
    if matches!(
        catalog.metadata.coverage_status.as_str(),
        "partial" | "cancelled" | "scope_changed"
    ) {
        vec![ProjectCatalogApplicationWarning {
            warning_code: "PROJECT_CATALOG_COVERAGE_INCOMPLETE".to_string(),
            message:
                "The latest Project scan had incomplete coverage; prior records were retained."
                    .to_string(),
        }]
    } else {
        Vec::new()
    }
}

fn failed_list(code: &str, stage: &str, message: &str) -> ProjectCatalogListResult {
    let mut result = empty_list();
    result.errors.push(ProjectCatalogApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    });
    result
}
