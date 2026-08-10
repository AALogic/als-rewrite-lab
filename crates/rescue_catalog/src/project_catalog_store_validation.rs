use crate::{
    ProjectCatalogSnapshot, ProjectCatalogStoreError, ProjectCatalogStoreRequest,
    StoredProjectCatalog, PROJECT_CATALOG_IDENTITY_POLICY, PROJECT_CATALOG_STORAGE_SCHEMA_VERSION,
    PROJECT_CATALOG_VERSION,
};
use std::collections::HashSet;

pub(crate) fn validate_store_request(
    request: &ProjectCatalogStoreRequest,
) -> Result<(), ProjectCatalogStoreError> {
    if !request.store_path.is_absolute() || request.store_path.file_name().is_none() {
        return Err(store_error(
            "CATALOG_STORE_PATH_INVALID",
            "The private catalog store path must be an absolute file path.",
        ));
    }
    if request.coverage_scope_id.trim().is_empty() {
        return Err(store_error(
            "CATALOG_STORE_INPUT_INVALID",
            "The catalog coverage scope ID must not be empty.",
        ));
    }
    validate_snapshot(&request.snapshot)
}

pub(crate) fn validate_snapshot(
    snapshot: &ProjectCatalogSnapshot,
) -> Result<(), ProjectCatalogStoreError> {
    let metadata = &snapshot.metadata;
    if metadata.catalog_version != PROJECT_CATALOG_VERSION
        || metadata.identity_policy_version != PROJECT_CATALOG_IDENTITY_POLICY
    {
        return Err(store_error(
            "CATALOG_STORE_INPUT_INVALID",
            "The ProjectCatalogSnapshot contract version is unsupported.",
        ));
    }
    if !matches!(
        metadata.build_status.as_str(),
        "complete" | "complete_with_ambiguity" | "partial"
    ) || !snapshot.errors.is_empty()
        || metadata.error_count != 0
    {
        return Err(store_error(
            "CATALOG_STORE_INPUT_INVALID",
            "Only a successful ProjectCatalogSnapshot can be persisted.",
        ));
    }
    validate_snapshot_counts(snapshot)?;
    validate_snapshot_records(snapshot)
}

fn validate_snapshot_counts(
    snapshot: &ProjectCatalogSnapshot,
) -> Result<(), ProjectCatalogStoreError> {
    let metadata = &snapshot.metadata;
    let backup_count = snapshot
        .live_sets
        .iter()
        .filter(|set| set.location_kind.starts_with("backup"))
        .count();
    if metadata.snapshot_id.trim().is_empty()
        || metadata.source_scan_run_id.trim().is_empty()
        || metadata.project_folder_count != snapshot.project_folders.len()
        || metadata.live_set_count != snapshot.live_sets.len()
        || metadata.backup_set_count != backup_count
        || metadata.main_set_count != snapshot.live_sets.len().saturating_sub(backup_count)
        || metadata.warning_count != snapshot.warnings.len()
    {
        return Err(store_error(
            "CATALOG_STORE_INPUT_INVALID",
            "ProjectCatalogSnapshot metadata is inconsistent with its records.",
        ));
    }
    Ok(())
}

fn validate_snapshot_records(
    snapshot: &ProjectCatalogSnapshot,
) -> Result<(), ProjectCatalogStoreError> {
    let folder_ids = unique_ids(
        snapshot
            .project_folders
            .iter()
            .map(|folder| folder.project_folder_id.as_str()),
    )?;
    let set_ids = unique_ids(
        snapshot
            .live_sets
            .iter()
            .map(|set| set.live_set_id.as_str()),
    )?;
    for folder in &snapshot.project_folders {
        for set_id in folder.main_set_ids.iter().chain(&folder.backup_set_ids) {
            if !set_ids.contains(set_id.as_str()) {
                return Err(invalid_reference());
            }
        }
    }
    for set in &snapshot.live_sets {
        if set
            .project_folder_id
            .as_ref()
            .is_some_and(|id| !folder_ids.contains(id.as_str()))
            || set
                .candidate_project_folder_ids
                .iter()
                .any(|id| !folder_ids.contains(id.as_str()))
        {
            return Err(invalid_reference());
        }
    }
    Ok(())
}

pub(crate) fn validate_stored_catalog(
    catalog: &StoredProjectCatalog,
) -> Result<(), ProjectCatalogStoreError> {
    if catalog.metadata.storage_schema_version != PROJECT_CATALOG_STORAGE_SCHEMA_VERSION {
        return Err(store_error(
            "CATALOG_STORE_SCHEMA_UNSUPPORTED",
            "The private catalog storage schema is unsupported.",
        ));
    }
    if catalog.metadata.revision == 0
        || catalog.metadata.coverage_scope_id.trim().is_empty()
        || catalog.metadata.last_snapshot_id.trim().is_empty()
        || catalog.metadata.last_scan_run_id.trim().is_empty()
        || catalog.metadata.project_folder_count != catalog.project_folders.len()
        || catalog.metadata.live_set_count != catalog.live_sets.len()
    {
        return Err(invalid_state());
    }
    validate_stored_records(catalog)?;
    validate_freshness_counts(catalog)
}

fn validate_stored_records(catalog: &StoredProjectCatalog) -> Result<(), ProjectCatalogStoreError> {
    if !is_sorted(
        catalog
            .project_folders
            .iter()
            .map(|stored| stored.record.project_folder_id.as_str()),
    ) || !is_sorted(
        catalog
            .live_sets
            .iter()
            .map(|stored| stored.record.live_set_id.as_str()),
    ) {
        return Err(invalid_state());
    }
    let folder_ids = unique_ids(
        catalog
            .project_folders
            .iter()
            .map(|stored| stored.record.project_folder_id.as_str()),
    )?;
    let set_ids = unique_ids(
        catalog
            .live_sets
            .iter()
            .map(|stored| stored.record.live_set_id.as_str()),
    )?;
    for stored in &catalog.project_folders {
        validate_freshness(&stored.freshness_status, &stored.last_observed_scan_run_id)?;
        if stored
            .record
            .main_set_ids
            .iter()
            .chain(&stored.record.backup_set_ids)
            .any(|id| !set_ids.contains(id.as_str()))
        {
            return Err(invalid_state());
        }
    }
    for stored in &catalog.live_sets {
        validate_freshness(&stored.freshness_status, &stored.last_observed_scan_run_id)?;
        if stored
            .record
            .project_folder_id
            .as_ref()
            .is_some_and(|id| !folder_ids.contains(id.as_str()))
        {
            return Err(invalid_state());
        }
    }
    Ok(())
}

fn validate_freshness_counts(
    catalog: &StoredProjectCatalog,
) -> Result<(), ProjectCatalogStoreError> {
    let folders = &catalog.project_folders;
    let sets = &catalog.live_sets;
    let metadata = &catalog.metadata;
    if metadata.observed_project_folder_count != count_folders(folders, "observed_in_latest_scan")
        || metadata.observed_live_set_count != count_sets(sets, "observed_in_latest_scan")
        || metadata.retained_project_folder_count
            != count_folders(folders, "retained_from_prior_scan")
        || metadata.retained_live_set_count != count_sets(sets, "retained_from_prior_scan")
        || metadata.stale_project_folder_count
            != count_folders(folders, "not_observed_in_latest_complete_scan")
        || metadata.stale_live_set_count != count_sets(sets, "not_observed_in_latest_complete_scan")
    {
        return Err(invalid_state());
    }
    Ok(())
}

fn unique_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
) -> Result<HashSet<&'a str>, ProjectCatalogStoreError> {
    let mut unique = HashSet::new();
    for id in ids {
        if id.is_empty() || !unique.insert(id) {
            return Err(invalid_state());
        }
    }
    Ok(unique)
}

fn is_sorted<'a>(ids: impl Iterator<Item = &'a str>) -> bool {
    let values = ids.collect::<Vec<_>>();
    values.windows(2).all(|pair| pair[0] <= pair[1])
}

fn validate_freshness(status: &str, scan_run_id: &str) -> Result<(), ProjectCatalogStoreError> {
    if scan_run_id.trim().is_empty()
        || !matches!(
            status,
            "observed_in_latest_scan"
                | "retained_from_prior_scan"
                | "not_observed_in_latest_complete_scan"
        )
    {
        return Err(invalid_state());
    }
    Ok(())
}

fn count_folders(records: &[crate::StoredProjectFolderRecord], status: &str) -> usize {
    records
        .iter()
        .filter(|record| record.freshness_status == status)
        .count()
}

fn count_sets(records: &[crate::StoredLiveSetRecord], status: &str) -> usize {
    records
        .iter()
        .filter(|record| record.freshness_status == status)
        .count()
}

fn invalid_reference() -> ProjectCatalogStoreError {
    store_error(
        "CATALOG_STORE_INPUT_INVALID",
        "ProjectCatalogSnapshot contains an invalid record reference.",
    )
}

fn invalid_state() -> ProjectCatalogStoreError {
    store_error(
        "CATALOG_STORE_STATE_INVALID",
        "The private catalog state is internally inconsistent.",
    )
}

pub(crate) fn store_error(error_code: &str, message: &str) -> ProjectCatalogStoreError {
    ProjectCatalogStoreError {
        error_code: error_code.to_string(),
        message: message.to_string(),
    }
}
