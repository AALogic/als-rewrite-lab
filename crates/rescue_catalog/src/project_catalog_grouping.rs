use crate::{
    project_catalog_identity, ALSFileObservation, LiveSetRecord, ProjectCatalogError,
    ProjectFolderRecord, ProjectMarkerObservation, ProjectScanResult,
};
use std::path::Path;

pub(crate) fn build_records(
    scan: &ProjectScanResult,
) -> Result<(Vec<ProjectFolderRecord>, Vec<LiveSetRecord>), ProjectCatalogError> {
    let mut folders = scan
        .project_markers
        .iter()
        .map(folder_record)
        .collect::<Result<Vec<_>, _>>()?;
    let mut sets = scan
        .als_files
        .iter()
        .map(|observation| set_record(observation, &folders))
        .collect::<Result<Vec<_>, _>>()?;

    assign_sets_to_folders(&mut folders, &sets);
    folders.sort_by(|left, right| left.project_folder_id.cmp(&right.project_folder_id));
    sets.sort_by(|left, right| left.live_set_id.cmp(&right.live_set_id));
    Ok((folders, sets))
}

fn folder_record(
    marker: &ProjectMarkerObservation,
) -> Result<ProjectFolderRecord, ProjectCatalogError> {
    Ok(ProjectFolderRecord {
        project_folder_id: project_catalog_identity::project_folder_id(
            &marker.project_root_candidate,
        )?,
        source_root: marker.source_root.clone(),
        native_root_path: marker.project_root_candidate.clone(),
        marker_path: marker.marker_path.clone(),
        display_name: path_name(&marker.project_root_candidate)?,
        structural_status: "exact_marker_observed".to_string(),
        main_set_ids: Vec::new(),
        backup_set_ids: Vec::new(),
        latest_modified_time_unix_ms: None,
    })
}

fn set_record(
    observation: &ALSFileObservation,
    folders: &[ProjectFolderRecord],
) -> Result<LiveSetRecord, ProjectCatalogError> {
    let mut candidates: Vec<&ProjectFolderRecord> = folders
        .iter()
        .filter(|folder| {
            observation
                .native_path
                .starts_with(&folder.native_root_path)
        })
        .collect();
    candidates.sort_by(|left, right| left.project_folder_id.cmp(&right.project_folder_id));
    let candidate_ids = candidates
        .iter()
        .map(|folder| folder.project_folder_id.clone())
        .collect::<Vec<_>>();
    let backup = is_backup_location(observation, &candidates);
    let (association_status, project_folder_id, location_kind) =
        association(observation, &candidates, backup);

    Ok(LiveSetRecord {
        live_set_id: project_catalog_identity::live_set_id(&observation.native_path)?,
        source_root: observation.source_root.clone(),
        native_als_path: observation.native_path.clone(),
        relative_path: observation.relative_path.clone(),
        display_name: set_display_name(observation)?,
        file_size: observation.file_size,
        modified_time_unix_ms: observation.modified_time_unix_ms,
        location_kind,
        association_status,
        project_folder_id,
        candidate_project_folder_ids: candidate_ids,
        observation_fingerprint: project_catalog_identity::observation_fingerprint(
            &observation.native_path,
            observation.file_size,
            observation.modified_time_unix_ms,
        )?,
    })
}

fn association(
    observation: &ALSFileObservation,
    candidates: &[&ProjectFolderRecord],
    backup: bool,
) -> (String, Option<String>, String) {
    match candidates {
        [] => (
            "ungrouped".to_string(),
            None,
            if backup {
                "backup_ungrouped"
            } else {
                "ungrouped"
            }
            .to_string(),
        ),
        [folder] => (
            "structurally_associated".to_string(),
            Some(folder.project_folder_id.clone()),
            associated_location(observation, folder, backup).to_string(),
        ),
        _ => (
            "ambiguous_project_context".to_string(),
            None,
            if backup {
                "backup_ambiguous_project_context"
            } else {
                "ambiguous_project_context"
            }
            .to_string(),
        ),
    }
}

fn associated_location(
    observation: &ALSFileObservation,
    folder: &ProjectFolderRecord,
    backup: bool,
) -> &'static str {
    if backup {
        "backup"
    } else if observation.native_path.parent() == Some(folder.native_root_path.as_path()) {
        "project_root"
    } else {
        "project_subdirectory"
    }
}

fn assign_sets_to_folders(folders: &mut [ProjectFolderRecord], sets: &[LiveSetRecord]) {
    for folder in folders {
        let associated = sets
            .iter()
            .filter(|set| set.project_folder_id.as_deref() == Some(&folder.project_folder_id));
        for set in associated {
            if set.location_kind == "backup" {
                folder.backup_set_ids.push(set.live_set_id.clone());
            } else {
                folder.main_set_ids.push(set.live_set_id.clone());
            }
            folder.latest_modified_time_unix_ms = max_time(
                folder.latest_modified_time_unix_ms,
                set.modified_time_unix_ms,
            );
        }
        folder.main_set_ids.sort();
        folder.backup_set_ids.sort();
    }
}

fn max_time(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn has_backup_component(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "Backup")
}

fn is_backup_location(
    observation: &ALSFileObservation,
    candidates: &[&ProjectFolderRecord],
) -> bool {
    if candidates.is_empty() {
        return has_backup_component(&observation.relative_path);
    }
    candidates.iter().all(|folder| {
        observation
            .native_path
            .strip_prefix(&folder.native_root_path)
            .is_ok_and(has_backup_component)
    })
}

fn path_name(path: &Path) -> Result<String, ProjectCatalogError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ProjectCatalogError {
            error_code: "CATALOG_PATH_NOT_UTF8".to_string(),
            message: "A catalog display name cannot be represented exactly.".to_string(),
        })
}

fn set_display_name(observation: &ALSFileObservation) -> Result<String, ProjectCatalogError> {
    observation
        .native_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ProjectCatalogError {
            error_code: "CATALOG_PATH_NOT_UTF8".to_string(),
            message: "A Live Set display name cannot be represented exactly.".to_string(),
        })
}
