use crate::{
    ProjectCatalogSnapshot, ProjectCatalogStoreMetadata, ProjectCatalogStoreWarning,
    StoredLiveSetRecord, StoredProjectCatalog, StoredProjectFolderRecord,
    PROJECT_CATALOG_STORAGE_SCHEMA_VERSION,
};
use std::collections::BTreeMap;

pub(crate) fn merge_snapshot(
    prior: Option<&StoredProjectCatalog>,
    coverage_scope_id: &str,
    snapshot: &ProjectCatalogSnapshot,
) -> (StoredProjectCatalog, Vec<ProjectCatalogStoreWarning>) {
    let source_partial = matches!(
        snapshot.metadata.source_scan_status.as_str(),
        "partial" | "cancelled"
    );
    let scope_changed =
        prior.is_some_and(|catalog| catalog.metadata.coverage_scope_id != coverage_scope_id);
    let partial = source_partial || scope_changed;
    let scan_run_id = snapshot.metadata.source_scan_run_id.as_str();
    let mut folders = prior_folders(prior, partial);
    let mut sets = prior_sets(prior, partial);

    for record in &snapshot.project_folders {
        let record = if partial {
            merge_partial_folder(folders.get(&record.project_folder_id), record)
        } else {
            record.clone()
        };
        folders.insert(
            record.project_folder_id.clone(),
            StoredProjectFolderRecord {
                record,
                freshness_status: "observed_in_latest_scan".to_string(),
                last_observed_scan_run_id: scan_run_id.to_string(),
            },
        );
    }
    for record in &snapshot.live_sets {
        sets.insert(
            record.live_set_id.clone(),
            StoredLiveSetRecord {
                record: record.clone(),
                freshness_status: "observed_in_latest_scan".to_string(),
                last_observed_scan_run_id: scan_run_id.to_string(),
            },
        );
    }
    let project_folders = folders.into_values().collect::<Vec<_>>();
    let live_sets = sets.into_values().collect::<Vec<_>>();
    let revision = prior.map_or(1, |catalog| catalog.metadata.revision.saturating_add(1));
    let metadata = metadata(
        snapshot,
        coverage_scope_id,
        scope_changed,
        revision,
        &project_folders,
        &live_sets,
    );
    let warnings = coverage_warnings(&snapshot.metadata.source_scan_status, scope_changed);
    (
        StoredProjectCatalog {
            metadata,
            project_folders,
            live_sets,
            catalog_warnings: snapshot.warnings.clone(),
        },
        warnings,
    )
}

pub(crate) fn semantically_unchanged(
    prior: &StoredProjectCatalog,
    candidate: &StoredProjectCatalog,
) -> bool {
    let mut normalized = candidate.clone();
    normalized.metadata.revision = prior.metadata.revision;
    &normalized == prior
}

fn prior_folders(
    prior: Option<&StoredProjectCatalog>,
    partial: bool,
) -> BTreeMap<String, StoredProjectFolderRecord> {
    prior
        .into_iter()
        .flat_map(|catalog| catalog.project_folders.iter())
        .map(|stored| {
            let mut stored = stored.clone();
            stored.freshness_status = absent_status(&stored.freshness_status, partial);
            (stored.record.project_folder_id.clone(), stored)
        })
        .collect()
}

fn prior_sets(
    prior: Option<&StoredProjectCatalog>,
    partial: bool,
) -> BTreeMap<String, StoredLiveSetRecord> {
    prior
        .into_iter()
        .flat_map(|catalog| catalog.live_sets.iter())
        .map(|stored| {
            let mut stored = stored.clone();
            stored.freshness_status = absent_status(&stored.freshness_status, partial);
            (stored.record.live_set_id.clone(), stored)
        })
        .collect()
}

fn absent_status(current: &str, partial: bool) -> String {
    if partial && current == "not_observed_in_latest_complete_scan" {
        current.to_string()
    } else if partial {
        "retained_from_prior_scan".to_string()
    } else {
        "not_observed_in_latest_complete_scan".to_string()
    }
}

fn merge_partial_folder(
    prior: Option<&StoredProjectFolderRecord>,
    incoming: &crate::ProjectFolderRecord,
) -> crate::ProjectFolderRecord {
    let Some(prior) = prior else {
        return incoming.clone();
    };
    let mut merged = incoming.clone();
    merged
        .main_set_ids
        .extend(prior.record.main_set_ids.clone());
    merged
        .backup_set_ids
        .extend(prior.record.backup_set_ids.clone());
    merged.main_set_ids.sort();
    merged.main_set_ids.dedup();
    merged.backup_set_ids.sort();
    merged.backup_set_ids.dedup();
    merged.latest_modified_time_unix_ms = max_time(
        merged.latest_modified_time_unix_ms,
        prior.record.latest_modified_time_unix_ms,
    );
    merged
}

fn metadata(
    snapshot: &ProjectCatalogSnapshot,
    coverage_scope_id: &str,
    scope_changed: bool,
    revision: u64,
    folders: &[StoredProjectFolderRecord],
    sets: &[StoredLiveSetRecord],
) -> ProjectCatalogStoreMetadata {
    ProjectCatalogStoreMetadata {
        storage_schema_version: PROJECT_CATALOG_STORAGE_SCHEMA_VERSION.to_string(),
        revision,
        coverage_scope_id: coverage_scope_id.to_string(),
        last_snapshot_id: snapshot.metadata.snapshot_id.clone(),
        last_scan_run_id: snapshot.metadata.source_scan_run_id.clone(),
        last_scan_status: snapshot.metadata.source_scan_status.clone(),
        coverage_status: if scope_changed && snapshot.metadata.source_scan_status == "complete" {
            "scope_changed".to_string()
        } else {
            snapshot.metadata.source_scan_status.clone()
        },
        project_folder_count: folders.len(),
        live_set_count: sets.len(),
        observed_project_folder_count: count_folders(folders, "observed_in_latest_scan"),
        observed_live_set_count: count_sets(sets, "observed_in_latest_scan"),
        retained_project_folder_count: count_folders(folders, "retained_from_prior_scan"),
        retained_live_set_count: count_sets(sets, "retained_from_prior_scan"),
        stale_project_folder_count: count_folders(folders, "not_observed_in_latest_complete_scan"),
        stale_live_set_count: count_sets(sets, "not_observed_in_latest_complete_scan"),
    }
}

fn count_folders(records: &[StoredProjectFolderRecord], status: &str) -> usize {
    records
        .iter()
        .filter(|record| record.freshness_status == status)
        .count()
}

fn count_sets(records: &[StoredLiveSetRecord], status: &str) -> usize {
    records
        .iter()
        .filter(|record| record.freshness_status == status)
        .count()
}

fn max_time(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn coverage_warnings(status: &str, scope_changed: bool) -> Vec<ProjectCatalogStoreWarning> {
    let warning = if scope_changed && status == "complete" {
        Some((
            "CATALOG_STORE_SCOPE_CHANGED_RETAINED",
            "Prior catalog records were retained because the complete scan scope changed.",
        ))
    } else {
        match status {
            "partial" => Some((
                "CATALOG_STORE_PARTIAL_COVERAGE_RETAINED",
                "Prior catalog records were retained because the latest scan was partial.",
            )),
            "cancelled" => Some((
                "CATALOG_STORE_CANCELLED_COVERAGE_RETAINED",
                "Prior catalog records were retained because the latest scan was cancelled.",
            )),
            _ => None,
        }
    };
    warning
        .into_iter()
        .map(|(warning_code, message)| ProjectCatalogStoreWarning {
            warning_code: warning_code.to_string(),
            message: message.to_string(),
        })
        .collect()
}
