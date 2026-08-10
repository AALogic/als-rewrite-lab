use rescue_application::{
    list_project_catalog, refresh_project_catalog, resolve_project_selection,
    ProjectCatalogListRequest, ProjectCatalogListResult, ProjectCatalogRefreshRequest,
    ProjectSelection, ProjectSelectionRequest, ProjectSelectionResult,
};
use rescue_catalog::{
    build_project_catalog, store_project_catalog, ALSFileObservation, ProjectCatalogBuildRequest,
    ProjectCatalogSnapshot, ProjectCatalogStoreRequest, ProjectMarkerObservation,
    ProjectScanMetadata, ProjectScanResult, PROJECT_SCANNER_VERSION, PROJECT_SCAN_TRAVERSAL_POLICY,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    scan_root: PathBuf,
    store_path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temporary application fixture");
        Self {
            scan_root: temp.path().join("scan"),
            store_path: temp.path().join("private/catalog.json"),
            _temp: temp,
        }
    }

    fn file(&self, relative: &str) -> PathBuf {
        let path = self.scan_root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture parent");
        fs::write(&path, b"catalog scan must not parse ALS").expect("fixture file");
        path
    }

    fn marker(&self, relative: &str) {
        fs::create_dir_all(self.scan_root.join(relative).join("Ableton Project Info"))
            .expect("fixture marker");
    }

    fn list(
        &self,
        include_backups: bool,
        include_stale: bool,
    ) -> rescue_application::ProjectCatalogListResult {
        list_project_catalog(&ProjectCatalogListRequest {
            store_path: self.store_path.clone(),
            include_backups,
            include_stale,
        })
    }

    fn store(&self, snapshot: ProjectCatalogSnapshot) {
        let result = store_project_catalog(&ProjectCatalogStoreRequest {
            store_path: self.store_path.clone(),
            coverage_scope_id: "scope-fixture".to_string(),
            snapshot,
        });
        assert!(
            result.errors.is_empty(),
            "store fixture: {:?}",
            result.errors
        );
    }
}

fn observed_set(id: &str, root: &Path, relative: &str, size: u64) -> ALSFileObservation {
    let relative_path = PathBuf::from(relative);
    ALSFileObservation {
        observation_id: id.to_string(),
        source_root: root.to_path_buf(),
        native_path: root.join(&relative_path),
        relative_path: relative_path.clone(),
        filename: relative_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("fixture.als")
            .to_string(),
        extension: "als".to_string(),
        file_size: size,
        modified_time_unix_ms: Some(size),
        entry_kind: "file".to_string(),
        observation_status: "observed".to_string(),
    }
}

fn observed_marker(id: &str, root: &Path, relative: &str) -> ProjectMarkerObservation {
    let project_root = root.join(relative);
    ProjectMarkerObservation {
        marker_observation_id: id.to_string(),
        source_root: root.to_path_buf(),
        project_root_candidate: project_root.clone(),
        marker_path: project_root.join("Ableton Project Info"),
        relative_marker_path: PathBuf::from(relative).join("Ableton Project Info"),
        marker_name: "Ableton Project Info".to_string(),
        entry_kind: "directory".to_string(),
        observation_status: "observed".to_string(),
    }
}

fn snapshot(
    fixture: &Fixture,
    id: &str,
    run: &str,
    status: &str,
    sets: Vec<ALSFileObservation>,
    markers: Vec<ProjectMarkerObservation>,
) -> ProjectCatalogSnapshot {
    let metadata = ProjectScanMetadata {
        scanner_version: PROJECT_SCANNER_VERSION.to_string(),
        traversal_policy_version: PROJECT_SCAN_TRAVERSAL_POLICY.to_string(),
        scan_run_id: run.to_string(),
        scan_status: status.to_string(),
        requested_root_count: 1,
        accepted_root_count: 1,
        directories_visited: 10,
        entries_visited: sets.len() + markers.len(),
        als_file_count: sets.len(),
        project_marker_count: markers.len(),
        skipped_symlink_count: 0,
        skipped_excluded_count: 0,
        warning_count: 0,
        error_count: 0,
    };
    build_project_catalog(ProjectCatalogBuildRequest {
        snapshot_id: format!("{id}-{}", fixture.scan_root.display()),
        scan_result: ProjectScanResult {
            metadata,
            als_files: sets,
            project_markers: markers,
            warnings: Vec::new(),
            errors: Vec::new(),
        },
    })
}

fn complete_catalog(fixture: &Fixture, id: &str, run: &str) -> ProjectCatalogSnapshot {
    snapshot(
        fixture,
        id,
        run,
        "complete",
        vec![
            observed_set("main", &fixture.scan_root, "Project/Main.als", 100),
            observed_set("alt", &fixture.scan_root, "Project/Alternate.als", 90),
            observed_set("backup", &fixture.scan_root, "Project/Backup/Old.als", 80),
            observed_set("loose", &fixture.scan_root, "Loose/orphan.als", 70),
        ],
        vec![observed_marker("marker", &fixture.scan_root, "Project")],
    )
}

#[test]
fn refresh_composes_scan_build_store_and_list() {
    let fixture = Fixture::new();
    fixture.file("Project/Main.als");
    fixture.marker("Project");
    let result = refresh_project_catalog(&ProjectCatalogRefreshRequest {
        request_id: "refresh-1".to_string(),
        roots: vec![fixture.scan_root.clone()],
        excluded_roots: Vec::new(),
        store_path: fixture.store_path.clone(),
        max_entries: 1_000,
        max_depth: None,
        include_backups: false,
        include_stale: false,
    });

    assert_eq!(result.refresh_status, "complete");
    assert_eq!(result.scan_status, "complete");
    assert_eq!(result.store_status, "stored");
    assert_eq!(result.catalog.items.len(), 1);
    assert!(result.errors.is_empty());
    assert!(fixture.store_path.exists());
}

#[test]
fn default_list_hides_backups_and_stale_records() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let second = snapshot(
        &fixture,
        "second",
        "scan-2",
        "complete",
        vec![observed_set(
            "main-2",
            &fixture.scan_root,
            "Project/Main.als",
            101,
        )],
        vec![observed_marker("marker-2", &fixture.scan_root, "Project")],
    );
    fixture.store(second);
    let hidden = fixture.list(false, false);
    let expanded = fixture.list(true, true);

    assert_eq!(hidden.items.len(), 1);
    assert!(hidden.metadata.hidden_backup_count >= 1);
    assert!(hidden.metadata.hidden_stale_count >= 1);
    assert!(expanded.items.len() > hidden.items.len());
    assert!(expanded
        .items
        .iter()
        .any(|item| item.selection_status == "stale_unavailable"));
}

#[test]
fn retained_partial_records_remain_visible_with_warning() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let partial = snapshot(
        &fixture,
        "partial",
        "scan-2",
        "partial",
        vec![observed_set(
            "main-2",
            &fixture.scan_root,
            "Project/Main.als",
            101,
        )],
        vec![observed_marker("marker-2", &fixture.scan_root, "Project")],
    );
    fixture.store(partial);
    let list = fixture.list(false, false);

    assert!(list.items.iter().any(|item| {
        item.display_name == "Alternate"
            && item.freshness_status == "retained_from_prior_scan"
            && item.selection_status == "selectable_with_warning"
    }));
}

#[test]
fn multiple_main_sets_remain_separate_items() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let list = fixture.list(false, false);
    let project_items = list
        .items
        .iter()
        .filter(|item| item.project_display_name.as_deref() == Some("Project"))
        .collect::<Vec<_>>();

    assert_eq!(project_items.len(), 2);
    assert_ne!(project_items[0].live_set_id, project_items[1].live_set_id);
}

#[test]
fn duplicate_display_names_never_merge() {
    let fixture = Fixture::new();
    let snapshot = snapshot(
        &fixture,
        "duplicates",
        "scan-1",
        "complete",
        vec![
            observed_set("a", &fixture.scan_root, "A/Project/Main.als", 100),
            observed_set("b", &fixture.scan_root, "B/Project/Main.als", 100),
        ],
        vec![
            observed_marker("ma", &fixture.scan_root, "A/Project"),
            observed_marker("mb", &fixture.scan_root, "B/Project"),
        ],
    );
    fixture.store(snapshot);
    let list = fixture.list(false, false);

    assert_eq!(list.items.len(), 2);
    assert!(list
        .items
        .iter()
        .all(|item| item.duplicate_display_name_count == 2));
    assert_ne!(list.items[0].group_id, list.items[1].group_id);
}

#[test]
fn ambiguous_and_ungrouped_sets_remain_explicit() {
    let fixture = Fixture::new();
    let snapshot = snapshot(
        &fixture,
        "contexts",
        "scan-1",
        "complete",
        vec![
            observed_set("amb", &fixture.scan_root, "Outer/Inner/Ambiguous.als", 100),
            observed_set("loose", &fixture.scan_root, "Loose/orphan.als", 90),
        ],
        vec![
            observed_marker("outer", &fixture.scan_root, "Outer"),
            observed_marker("inner", &fixture.scan_root, "Outer/Inner"),
        ],
    );
    fixture.store(snapshot);
    let list = fixture.list(false, false);

    assert!(list
        .groups
        .iter()
        .any(|group| group.group_kind == "ambiguous_project_context"));
    assert!(list
        .groups
        .iter()
        .any(|group| group.group_kind == "ungrouped_sets"));
    assert!(list
        .items
        .iter()
        .all(|item| item.selection_status == "selectable_with_warning"));
}

#[test]
fn project_list_contains_no_native_paths() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let json = serde_json::to_string(&fixture.list(true, true)).expect("list JSON");

    assert!(!json.contains(fixture.scan_root.to_string_lossy().as_ref()));
    assert!(!json.contains(fixture.store_path.to_string_lossy().as_ref()));
    assert!(!json.contains("native_als_path"));
    assert!(!json.contains("observation_fingerprint"));
}

fn catalog_selection_request(
    fixture: &Fixture,
    revision: u64,
    ids: Vec<String>,
) -> ProjectSelectionRequest {
    ProjectSelectionRequest {
        request_id: "selection-1".to_string(),
        store_path: fixture.store_path.clone(),
        catalog_revision: Some(revision),
        catalog_live_set_ids: ids,
        manual_als_paths: Vec::new(),
    }
}

#[test]
fn catalog_selection_requires_matching_revision() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let list = fixture.list(false, false);
    let id = list.items[0].live_set_id.clone();
    let missing = resolve_project_selection(&ProjectSelectionRequest {
        catalog_revision: None,
        ..catalog_selection_request(&fixture, list.metadata.catalog_revision, vec![id.clone()])
    });
    let stale = resolve_project_selection(&catalog_selection_request(
        &fixture,
        list.metadata.catalog_revision + 1,
        vec![id],
    ));

    assert_eq!(missing.selection_status, "failed");
    assert_eq!(stale.selection_status, "failed");
    assert!(missing
        .errors
        .iter()
        .any(|error| error.error_code == "PROJECT_SELECTION_REVISION_REQUIRED"));
    assert!(stale
        .errors
        .iter()
        .any(|error| error.error_code == "PROJECT_SELECTION_STALE_REVISION"));
}

#[test]
fn stale_catalog_item_cannot_be_selected() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    fixture.store(snapshot(
        &fixture,
        "second",
        "scan-2",
        "complete",
        vec![observed_set(
            "main-2",
            &fixture.scan_root,
            "Project/Main.als",
            101,
        )],
        vec![observed_marker("marker-2", &fixture.scan_root, "Project")],
    ));
    let list = fixture.list(true, true);
    let stale = list
        .items
        .iter()
        .find(|item| item.selection_status == "stale_unavailable")
        .expect("stale list item");
    let result = resolve_project_selection(&catalog_selection_request(
        &fixture,
        list.metadata.catalog_revision,
        vec![stale.live_set_id.clone()],
    ));

    assert_eq!(result.selection_status, "failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROJECT_SELECTION_STALE_ITEM"));
}

#[test]
fn manual_and_catalog_inputs_share_selection_contract() {
    let fixture = Fixture::new();
    let manual = fixture.file("Manual/Manual.als");
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let list = fixture.list(false, false);
    let result = resolve_project_selection(&ProjectSelectionRequest {
        request_id: "combined".to_string(),
        store_path: fixture.store_path.clone(),
        catalog_revision: Some(list.metadata.catalog_revision),
        catalog_live_set_ids: vec![list.items[0].live_set_id.clone()],
        manual_als_paths: vec![manual.clone()],
    });

    assert_eq!(result.selection_status, "resolved");
    assert_eq!(result.selections.len(), 2);
    assert!(result.selections.iter().any(|selection| {
        selection.selection_source == "catalog" && selection.live_set_id.is_some()
    }));
    assert!(result.selections.iter().any(|selection| {
        selection.selection_source == "manual"
            && selection.live_set_id.is_none()
            && selection.native_als_path == manual
    }));
}

#[test]
fn duplicate_selection_fails_closed() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let list = fixture.list(false, false);
    let id = list.items[0].live_set_id.clone();
    let duplicate_id = resolve_project_selection(&catalog_selection_request(
        &fixture,
        list.metadata.catalog_revision,
        vec![id.clone(), id],
    ));

    assert_eq!(duplicate_id.selection_status, "failed");
    assert!(duplicate_id
        .errors
        .iter()
        .any(|error| error.error_code == "PROJECT_SELECTION_DUPLICATE_ID"));
}

#[test]
fn catalog_application_diagnostics_are_path_free() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.store_path.parent().expect("store parent")).expect("store parent");
    fs::write(&fixture.store_path, b"corrupt").expect("corrupt store");
    let result = fixture.list(false, false);
    let diagnostics = format!("{:?}", result.errors);

    assert!(!diagnostics.contains(fixture.store_path.to_string_lossy().as_ref()));
    assert!(!diagnostics.contains(fixture.scan_root.to_string_lossy().as_ref()));
}

#[test]
fn fake_batch_consumer_receives_ordered_selections() {
    fn paths(selections: &[ProjectSelection]) -> Vec<&Path> {
        selections
            .iter()
            .map(|selection| selection.native_als_path.as_path())
            .collect()
    }

    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let list = fixture.list(false, false);
    let ids = list
        .items
        .iter()
        .map(|item| item.live_set_id.clone())
        .collect::<Vec<_>>();
    let result = resolve_project_selection(&catalog_selection_request(
        &fixture,
        list.metadata.catalog_revision,
        ids,
    ));

    assert_eq!(result.selection_status, "resolved");
    assert_eq!(paths(&result.selections).len(), list.items.len());
}

#[test]
fn project_catalog_wire_contract_is_stable() {
    let fixture = Fixture::new();
    fixture.store(complete_catalog(&fixture, "first", "scan-1"));
    let value = serde_json::to_value(fixture.list(false, false)).expect("wire JSON");
    let object = value.as_object().expect("list object");

    assert_eq!(object.len(), 5);
    for key in ["metadata", "groups", "items", "warnings", "errors"] {
        assert!(object.contains_key(key));
    }
    let item = object["items"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.as_object())
        .expect("wire item");
    assert!(!item.contains_key("native_als_path"));
    assert!(!item.contains_key("observation_fingerprint"));

    let shared_list: ProjectCatalogListResult = serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/project-catalog-list.json"
    ))
    .expect("shared Project catalog list fixture");
    let shared_selection: ProjectSelectionResult = serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/project-selection-result.json"
    ))
    .expect("shared Project selection fixture");
    assert_eq!(shared_list.items[0].live_set_id, "set-1");
    assert_eq!(
        shared_selection.selections[0].live_set_id.as_deref(),
        Some("set-1")
    );
}
