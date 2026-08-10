use rescue_catalog::{
    build_project_catalog, ALSFileObservation, ProjectCatalogBuildRequest, ProjectCatalogSnapshot,
    ProjectMarkerObservation, ProjectScanMetadata, ProjectScanResult, PROJECT_SCANNER_VERSION,
    PROJECT_SCAN_TRAVERSAL_POLICY,
};
use std::path::{Path, PathBuf};

fn fixture_root(name: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\fixture").join(name)
    } else {
        PathBuf::from("/fixture").join(name)
    }
}

fn set(id: &str, source_root: &Path, relative: &str, modified: u64) -> ALSFileObservation {
    let relative_path = PathBuf::from(relative);
    let filename = relative_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("fixture.als")
        .to_string();
    ALSFileObservation {
        observation_id: id.to_string(),
        source_root: source_root.to_path_buf(),
        native_path: source_root.join(&relative_path),
        relative_path,
        filename,
        extension: "als".to_string(),
        file_size: 1_024,
        modified_time_unix_ms: Some(modified),
        entry_kind: "file".to_string(),
        observation_status: "observed".to_string(),
    }
}

fn marker(id: &str, source_root: &Path, relative_root: &str) -> ProjectMarkerObservation {
    let project_root_candidate = source_root.join(relative_root);
    ProjectMarkerObservation {
        marker_observation_id: id.to_string(),
        source_root: source_root.to_path_buf(),
        marker_path: project_root_candidate.join("Ableton Project Info"),
        relative_marker_path: PathBuf::from(relative_root).join("Ableton Project Info"),
        project_root_candidate,
        marker_name: "Ableton Project Info".to_string(),
        entry_kind: "directory".to_string(),
        observation_status: "observed".to_string(),
    }
}

fn scan_result(
    status: &str,
    als_files: Vec<ALSFileObservation>,
    project_markers: Vec<ProjectMarkerObservation>,
) -> ProjectScanResult {
    let metadata = ProjectScanMetadata {
        scanner_version: PROJECT_SCANNER_VERSION.to_string(),
        traversal_policy_version: PROJECT_SCAN_TRAVERSAL_POLICY.to_string(),
        scan_run_id: "scan-fixture".to_string(),
        scan_status: status.to_string(),
        requested_root_count: 1,
        accepted_root_count: 1,
        directories_visited: 10,
        entries_visited: als_files.len() + project_markers.len(),
        als_file_count: als_files.len(),
        project_marker_count: project_markers.len(),
        skipped_symlink_count: 0,
        skipped_excluded_count: 0,
        warning_count: 0,
        error_count: 0,
    };
    ProjectScanResult {
        metadata,
        als_files,
        project_markers,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn build(scan_result: ProjectScanResult) -> ProjectCatalogSnapshot {
    build_project_catalog(ProjectCatalogBuildRequest {
        snapshot_id: "catalog-fixture".to_string(),
        scan_result,
    })
}

#[test]
fn standard_project_builds_physical_catalog() {
    let root = fixture_root("standard");
    let result = build(scan_result(
        "complete",
        vec![set("set-1", &root, "Project/Main.als", 100)],
        vec![marker("marker-1", &root, "Project")],
    ));

    assert_eq!(result.metadata.build_status, "complete");
    assert_eq!(result.project_folders.len(), 1);
    assert_eq!(result.live_sets.len(), 1);
    assert_eq!(
        result.live_sets[0].association_status,
        "structurally_associated"
    );
    assert_eq!(result.live_sets[0].location_kind, "project_root");
    assert_eq!(
        result.project_folders[0].main_set_ids,
        vec![result.live_sets[0].live_set_id.clone()]
    );
}

#[test]
fn multiple_main_sets_are_retained_without_primary_selection() {
    let root = fixture_root("multiple");
    let result = build(scan_result(
        "complete",
        vec![
            set("set-a", &root, "Project/A.als", 100),
            set("set-b", &root, "Project/B.als", 200),
        ],
        vec![marker("marker", &root, "Project")],
    ));

    assert_eq!(result.metadata.main_set_count, 2);
    assert_eq!(result.project_folders[0].main_set_ids.len(), 2);
    assert_eq!(result.live_sets.len(), 2);
    assert!(!format!("{result:?}").contains("primary_set"));
}

#[test]
fn backup_set_is_retained_separately() {
    let root = fixture_root("backup");
    let result = build(scan_result(
        "complete",
        vec![
            set("main", &root, "Project/Main.als", 200),
            set("backup", &root, "Project/Backup/Main [old].als", 100),
        ],
        vec![marker("marker", &root, "Project")],
    ));

    let folder = &result.project_folders[0];
    assert_eq!(folder.main_set_ids.len(), 1);
    assert_eq!(folder.backup_set_ids.len(), 1);
    assert_eq!(result.metadata.backup_set_count, 1);
    assert!(result
        .live_sets
        .iter()
        .any(|set| set.location_kind == "backup"));
}

#[test]
fn backup_classification_requires_exact_component_below_project_root() {
    let root = fixture_root("backup-boundary");
    let result = build(scan_result(
        "complete",
        vec![
            set("root", &root, "Backup/Main.als", 200),
            set("plural", &root, "Backup/Backups/Old.als", 100),
        ],
        vec![marker("marker", &root, "Backup")],
    ));

    assert_eq!(result.metadata.backup_set_count, 0);
    assert!(result
        .live_sets
        .iter()
        .all(|set| set.location_kind != "backup"));
}

#[test]
fn standalone_set_remains_ungrouped() {
    let root = fixture_root("standalone");
    let result = build(scan_result(
        "complete",
        vec![set("orphan", &root, "Loose/orphan.als", 100)],
        Vec::new(),
    ));

    assert_eq!(result.live_sets[0].association_status, "ungrouped");
    assert_eq!(result.live_sets[0].location_kind, "ungrouped");
    assert!(result.live_sets[0].project_folder_id.is_none());
    assert_eq!(result.metadata.ungrouped_set_count, 1);
}

#[test]
fn same_display_names_at_different_paths_do_not_merge() {
    let first = fixture_root("same-a");
    let second = fixture_root("same-b");
    let result = build(scan_result(
        "complete",
        vec![
            set("a", &first, "Project/Main.als", 100),
            set("b", &second, "Project/Main.als", 100),
        ],
        vec![
            marker("ma", &first, "Project"),
            marker("mb", &second, "Project"),
        ],
    ));

    assert_eq!(result.project_folders.len(), 2);
    assert_eq!(result.live_sets.len(), 2);
    assert_ne!(
        result.project_folders[0].project_folder_id,
        result.project_folders[1].project_folder_id
    );
    assert_ne!(
        result.live_sets[0].live_set_id,
        result.live_sets[1].live_set_id
    );
}

#[test]
fn nested_marker_candidates_remain_ambiguous() {
    let root = fixture_root("nested");
    let result = build(scan_result(
        "complete",
        vec![set("nested", &root, "Outer/Inner/Ambiguous.als", 100)],
        vec![
            marker("outer", &root, "Outer"),
            marker("inner", &root, "Outer/Inner"),
        ],
    ));

    let live_set = &result.live_sets[0];
    assert_eq!(result.metadata.build_status, "complete_with_ambiguity");
    assert_eq!(live_set.association_status, "ambiguous_project_context");
    assert!(live_set.project_folder_id.is_none());
    assert_eq!(live_set.candidate_project_folder_ids.len(), 2);
    assert!(result
        .project_folders
        .iter()
        .all(|folder| folder.main_set_ids.is_empty()));
}

#[test]
fn marker_without_set_is_retained() {
    let root = fixture_root("marker-only");
    let result = build(scan_result(
        "complete",
        Vec::new(),
        vec![marker("empty", &root, "Empty Project")],
    ));

    assert_eq!(result.project_folders.len(), 1);
    assert!(result.project_folders[0].main_set_ids.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "CATALOG_MARKER_WITHOUT_SET"));
}

#[test]
fn partial_and_cancelled_scan_coverage_propagates() {
    for status in ["partial", "cancelled"] {
        let root = fixture_root(status);
        let result = build(scan_result(
            status,
            vec![set("set", &root, "Main.als", 100)],
            Vec::new(),
        ));
        assert_eq!(result.metadata.build_status, "partial");
        assert_eq!(result.metadata.source_scan_status, status);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.warning_code.contains("SOURCE_SCAN")));
    }
}

#[test]
fn failed_or_inconsistent_scan_is_rejected() {
    let failed = build(scan_result("failed", Vec::new(), Vec::new()));
    assert_eq!(failed.metadata.build_status, "failed");
    assert!(failed
        .errors
        .iter()
        .any(|error| error.error_code == "CATALOG_SOURCE_SCAN_FAILED"));

    let root = fixture_root("inconsistent");
    let mut inconsistent = scan_result(
        "complete",
        vec![set("set", &root, "Main.als", 100)],
        Vec::new(),
    );
    inconsistent.metadata.als_file_count = 9;
    let result = build(inconsistent);
    assert_eq!(result.metadata.build_status, "failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "CATALOG_SOURCE_COUNTS_INCONSISTENT"));
}

#[test]
fn unsupported_and_duplicate_scan_evidence_is_rejected() {
    let root = fixture_root("invalid-evidence");
    let mut unsupported = scan_result("complete", Vec::new(), Vec::new());
    unsupported.metadata.scanner_version = "99.0".to_string();
    let unsupported_result = build(unsupported);
    assert!(unsupported_result
        .errors
        .iter()
        .any(|error| { error.error_code == "CATALOG_UNSUPPORTED_SCANNER_VERSION" }));

    let first = set("duplicate", &root, "A.als", 100);
    let second_id = set("duplicate", &root, "B.als", 100);
    let duplicate_id = build(scan_result(
        "complete",
        vec![first.clone(), second_id],
        Vec::new(),
    ));
    assert!(duplicate_id
        .errors
        .iter()
        .any(|error| { error.error_code == "CATALOG_DUPLICATE_OBSERVATION_ID" }));

    let mut second_path = first.clone();
    second_path.observation_id = "different".to_string();
    let duplicate_path = build(scan_result(
        "complete",
        vec![first, second_path],
        Vec::new(),
    ));
    assert!(duplicate_path
        .errors
        .iter()
        .any(|error| { error.error_code == "CATALOG_DUPLICATE_NATIVE_PATH" }));
}

#[test]
fn catalog_output_is_deterministic() {
    let root = fixture_root("deterministic");
    let input = scan_result(
        "complete",
        vec![
            set("b", &root, "Project/B.als", 200),
            set("a", &root, "Project/A.als", 100),
        ],
        vec![marker("marker", &root, "Project")],
    );

    assert_eq!(build(input.clone()), build(input));
}

#[test]
fn path_ids_change_when_native_paths_change() {
    let first = fixture_root("identity-a");
    let second = fixture_root("identity-b");
    let a = build(scan_result(
        "complete",
        vec![set("same-upstream-id", &first, "Main.als", 100)],
        Vec::new(),
    ));
    let b = build(scan_result(
        "complete",
        vec![set("same-upstream-id", &second, "Main.als", 100)],
        Vec::new(),
    ));

    assert_ne!(a.live_sets[0].live_set_id, b.live_sets[0].live_set_id);
    assert_ne!(
        a.live_sets[0].observation_fingerprint,
        b.live_sets[0].observation_fingerprint
    );
}

#[test]
fn catalog_warnings_contain_no_native_paths() {
    let root = fixture_root("private-path");
    let result = build(scan_result(
        "partial",
        vec![set("secret", &root, "Secret Project/secret.als", 100)],
        Vec::new(),
    ));
    let warnings = format!("{:?}", result.warnings);

    assert!(!warnings.contains(root.to_str().unwrap_or("private-path")));
    assert!(!warnings.contains("Secret Project"));
    assert!(!warnings.contains("secret.als"));
}

#[test]
fn fake_catalog_store_uses_snapshot_contract() {
    #[derive(Default)]
    struct FakeStore {
        saved_snapshot_id: Option<String>,
        saved_set_count: usize,
    }
    impl FakeStore {
        fn replace(&mut self, snapshot: &ProjectCatalogSnapshot) {
            self.saved_snapshot_id = Some(snapshot.metadata.snapshot_id.clone());
            self.saved_set_count = snapshot.live_sets.len();
        }
    }

    let root = fixture_root("store");
    let snapshot = build(scan_result(
        "complete",
        vec![set("set", &root, "Main.als", 100)],
        Vec::new(),
    ));
    let mut store = FakeStore::default();
    store.replace(&snapshot);

    assert_eq!(store.saved_snapshot_id.as_deref(), Some("catalog-fixture"));
    assert_eq!(store.saved_set_count, 1);
}
