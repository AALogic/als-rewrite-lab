use crate::{
    project_catalog_grouping, project_catalog_validation, ProjectCatalogBuildRequest,
    ProjectCatalogError, ProjectCatalogMetadata, ProjectCatalogSnapshot, ProjectCatalogWarning,
    ProjectScanResult, PROJECT_CATALOG_IDENTITY_POLICY, PROJECT_CATALOG_VERSION,
};

pub(crate) fn build_project_catalog_impl(
    request: ProjectCatalogBuildRequest,
) -> ProjectCatalogSnapshot {
    if let Err(error) = project_catalog_validation::validate_request(&request) {
        return failed_snapshot(&request, error);
    }

    let (project_folders, live_sets) =
        match project_catalog_grouping::build_records(&request.scan_result) {
            Ok(records) => records,
            Err(error) => return failed_snapshot(&request, error),
        };
    let warnings = build_warnings(&request.scan_result, &project_folders, &live_sets);
    let ambiguous_set_count = live_sets
        .iter()
        .filter(|set| set.association_status == "ambiguous_project_context")
        .count();
    let ungrouped_set_count = live_sets
        .iter()
        .filter(|set| set.association_status == "ungrouped")
        .count();
    let backup_set_count = live_sets
        .iter()
        .filter(|set| set.location_kind.starts_with("backup"))
        .count();
    let build_status = match request.scan_result.metadata.scan_status.as_str() {
        "partial" | "cancelled" => "partial",
        _ if ambiguous_set_count > 0 => "complete_with_ambiguity",
        _ => "complete",
    };
    let metadata = ProjectCatalogMetadata {
        catalog_version: PROJECT_CATALOG_VERSION.to_string(),
        identity_policy_version: PROJECT_CATALOG_IDENTITY_POLICY.to_string(),
        snapshot_id: request.snapshot_id,
        source_scan_run_id: request.scan_result.metadata.scan_run_id,
        source_scan_status: request.scan_result.metadata.scan_status,
        build_status: build_status.to_string(),
        project_folder_count: project_folders.len(),
        live_set_count: live_sets.len(),
        main_set_count: live_sets.len() - backup_set_count,
        backup_set_count,
        ungrouped_set_count,
        ambiguous_set_count,
        source_warning_count: request.scan_result.metadata.warning_count,
        source_error_count: request.scan_result.metadata.error_count,
        warning_count: warnings.len(),
        error_count: 0,
    };
    ProjectCatalogSnapshot {
        metadata,
        project_folders,
        live_sets,
        warnings,
        errors: Vec::new(),
    }
}

fn build_warnings(
    scan: &ProjectScanResult,
    folders: &[crate::ProjectFolderRecord],
    sets: &[crate::LiveSetRecord],
) -> Vec<ProjectCatalogWarning> {
    let mut drafts: Vec<(&str, &str, Option<String>, Vec<String>)> = Vec::new();
    if scan.metadata.scan_status == "partial" {
        drafts.push((
            "CATALOG_SOURCE_SCAN_PARTIAL",
            "The catalog contains only the observations collected by a partial scan.",
            None,
            Vec::new(),
        ));
    } else if scan.metadata.scan_status == "cancelled" {
        drafts.push((
            "CATALOG_SOURCE_SCAN_CANCELLED",
            "The catalog contains only the observations collected before cancellation.",
            None,
            Vec::new(),
        ));
    }
    for set in sets {
        if set.association_status == "ungrouped" {
            drafts.push((
                "CATALOG_UNGROUPED_SET_RETAINED",
                "A Live Set was retained without a marker-backed project folder.",
                Some(set.live_set_id.clone()),
                Vec::new(),
            ));
        } else if set.association_status == "ambiguous_project_context" {
            drafts.push((
                "CATALOG_AMBIGUOUS_PROJECT_CONTEXT",
                "A Live Set has several structural project-folder candidates.",
                Some(set.live_set_id.clone()),
                set.candidate_project_folder_ids.clone(),
            ));
        }
    }
    for folder in folders {
        if folder.main_set_ids.is_empty() && folder.backup_set_ids.is_empty() {
            drafts.push((
                "CATALOG_MARKER_WITHOUT_SET",
                "A marker-backed project folder contains no uniquely associated Live Set.",
                None,
                vec![folder.project_folder_id.clone()],
            ));
        }
    }
    drafts
        .into_iter()
        .enumerate()
        .map(
            |(warning_id, (warning_code, message, related_set_id, folder_ids))| {
                ProjectCatalogWarning {
                    warning_id,
                    warning_code: warning_code.to_string(),
                    message: message.to_string(),
                    related_set_id,
                    related_project_folder_ids: folder_ids,
                }
            },
        )
        .collect()
}

fn failed_snapshot(
    request: &ProjectCatalogBuildRequest,
    catalog_error: ProjectCatalogError,
) -> ProjectCatalogSnapshot {
    let metadata = ProjectCatalogMetadata {
        catalog_version: PROJECT_CATALOG_VERSION.to_string(),
        identity_policy_version: PROJECT_CATALOG_IDENTITY_POLICY.to_string(),
        snapshot_id: request.snapshot_id.clone(),
        source_scan_run_id: request.scan_result.metadata.scan_run_id.clone(),
        source_scan_status: request.scan_result.metadata.scan_status.clone(),
        build_status: "failed".to_string(),
        project_folder_count: 0,
        live_set_count: 0,
        main_set_count: 0,
        backup_set_count: 0,
        ungrouped_set_count: 0,
        ambiguous_set_count: 0,
        source_warning_count: request.scan_result.metadata.warning_count,
        source_error_count: request.scan_result.metadata.error_count,
        warning_count: 0,
        error_count: 1,
    };
    ProjectCatalogSnapshot {
        metadata,
        project_folders: Vec::new(),
        live_sets: Vec::new(),
        warnings: Vec::new(),
        errors: vec![catalog_error],
    }
}
