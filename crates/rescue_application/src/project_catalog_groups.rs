use crate::{ProjectListGroup, ProjectListItem};
use rescue_catalog::{StoredLiveSetRecord, StoredProjectCatalog};
use std::collections::{BTreeMap, HashMap};

const UNGROUPED_GROUP_ID: &str = "catalog-group-ungrouped";
const AMBIGUOUS_GROUP_ID: &str = "catalog-group-ambiguous";

pub(crate) fn group_id(stored: &StoredLiveSetRecord) -> String {
    if let Some(id) = &stored.record.project_folder_id {
        id.clone()
    } else if stored.record.association_status == "ambiguous_project_context" {
        AMBIGUOUS_GROUP_ID.to_string()
    } else {
        UNGROUPED_GROUP_ID.to_string()
    }
}

pub(crate) fn build_groups(
    items: &[ProjectListItem],
    catalog: &StoredProjectCatalog,
) -> Vec<ProjectListGroup> {
    let folders = catalog
        .project_folders
        .iter()
        .map(|stored| (stored.record.project_folder_id.as_str(), stored))
        .collect::<HashMap<_, _>>();
    let mut grouped: BTreeMap<String, Vec<&ProjectListItem>> = BTreeMap::new();
    for item in items {
        grouped.entry(item.group_id.clone()).or_default().push(item);
    }
    let mut groups = grouped
        .into_iter()
        .map(|(id, items)| group_record(id, items, &folders))
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        left.display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
            .then(left.group_id.cmp(&right.group_id))
    });
    groups
}

fn group_record(
    id: String,
    items: Vec<&ProjectListItem>,
    folders: &HashMap<&str, &rescue_catalog::StoredProjectFolderRecord>,
) -> ProjectListGroup {
    let (display_name, group_kind) = group_identity(&id, folders);
    ProjectListGroup {
        group_id: id,
        display_name,
        group_kind,
        item_ids: items.iter().map(|item| item.item_id.clone()).collect(),
        main_set_count: items
            .iter()
            .filter(|item| !item.location_kind.starts_with("backup"))
            .count(),
        backup_set_count: items
            .iter()
            .filter(|item| item.location_kind.starts_with("backup"))
            .count(),
        freshness_status: aggregate_freshness(&items),
    }
}

fn group_identity(
    id: &str,
    folders: &HashMap<&str, &rescue_catalog::StoredProjectFolderRecord>,
) -> (String, String) {
    if id == UNGROUPED_GROUP_ID {
        ("Other Live Sets".to_string(), "ungrouped_sets".to_string())
    } else if id == AMBIGUOUS_GROUP_ID {
        (
            "Project context requires review".to_string(),
            "ambiguous_project_context".to_string(),
        )
    } else {
        (
            folders
                .get(id)
                .map(|stored| stored.record.display_name.clone())
                .unwrap_or_else(|| "Ableton Project".to_string()),
            "project_folder".to_string(),
        )
    }
}

fn aggregate_freshness(items: &[&ProjectListItem]) -> String {
    if items
        .iter()
        .any(|item| item.freshness_status == "observed_in_latest_scan")
    {
        "observed_in_latest_scan"
    } else if items
        .iter()
        .any(|item| item.freshness_status == "retained_from_prior_scan")
    {
        "retained_from_prior_scan"
    } else {
        "not_observed_in_latest_complete_scan"
    }
    .to_string()
}
