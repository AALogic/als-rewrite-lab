use rescue_catalog::{
    build_project_catalog, load_project_catalog, store_project_catalog, ALSFileObservation,
    ProjectCatalogBuildRequest, ProjectCatalogSnapshot, ProjectCatalogStoreRequest,
    ProjectMarkerObservation, ProjectScanMetadata, ProjectScanResult, StoredProjectCatalog,
    PROJECT_SCANNER_VERSION, PROJECT_SCAN_TRAVERSAL_POLICY,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct StoreFixture {
    _temp: TempDir,
    path: PathBuf,
    scan_root: PathBuf,
}

impl StoreFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temporary store directory");
        Self {
            path: temp.path().join("catalog-state.json"),
            scan_root: absolute_fixture_root(temp.path()).join("scan-root"),
            _temp: temp,
        }
    }

    fn store(&self, snapshot: ProjectCatalogSnapshot) -> rescue_catalog::ProjectCatalogStoreResult {
        store_project_catalog(&ProjectCatalogStoreRequest {
            store_path: self.path.clone(),
            coverage_scope_id: "scope-fixture".to_string(),
            snapshot,
        })
    }
}

fn absolute_fixture_root(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if cfg!(windows) {
        PathBuf::from(r"C:\fixture")
    } else {
        PathBuf::from("/fixture")
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
    snapshot_id: &str,
    scan_run_id: &str,
    status: &str,
    sets: Vec<ALSFileObservation>,
    markers: Vec<ProjectMarkerObservation>,
) -> ProjectCatalogSnapshot {
    let metadata = ProjectScanMetadata {
        scanner_version: PROJECT_SCANNER_VERSION.to_string(),
        traversal_policy_version: PROJECT_SCAN_TRAVERSAL_POLICY.to_string(),
        scan_run_id: scan_run_id.to_string(),
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
        snapshot_id: snapshot_id.to_string(),
        scan_result: ProjectScanResult {
            metadata,
            als_files: sets,
            project_markers: markers,
            warnings: Vec::new(),
            errors: Vec::new(),
        },
    })
}

fn two_projects(
    fixture: &StoreFixture,
    id: &str,
    run: &str,
    status: &str,
) -> ProjectCatalogSnapshot {
    snapshot(
        id,
        run,
        status,
        vec![
            observed_set("set-a", &fixture.scan_root, "Project A/Main.als", 100),
            observed_set("set-b", &fixture.scan_root, "Project B/Remix.als", 200),
        ],
        vec![
            observed_marker("marker-a", &fixture.scan_root, "Project A"),
            observed_marker("marker-b", &fixture.scan_root, "Project B"),
        ],
    )
}

fn catalog(result: &rescue_catalog::ProjectCatalogStoreResult) -> &StoredProjectCatalog {
    result.catalog.as_ref().expect("successful store catalog")
}

#[test]
fn first_snapshot_persists_and_round_trips() {
    let fixture = StoreFixture::new();
    let result = fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));

    assert_eq!(result.operation_status, "stored");
    assert!(result.errors.is_empty());
    assert_eq!(catalog(&result).metadata.revision, 1);
    assert_eq!(catalog(&result).live_sets.len(), 2);
    let loaded = load_project_catalog(&fixture.path);
    assert_eq!(loaded.load_status, "loaded");
    assert_eq!(loaded.catalog, result.catalog);
}

#[test]
fn repeated_identical_snapshot_is_idempotent() {
    let fixture = StoreFixture::new();
    let input = two_projects(&fixture, "snapshot-1", "scan-1", "complete");
    let first = fixture.store(input.clone());
    let bytes = fs::read(&fixture.path).expect("stored bytes");
    let second = fixture.store(input);

    assert_eq!(first.operation_status, "stored");
    assert_eq!(second.operation_status, "already_current");
    assert_eq!(catalog(&second).metadata.revision, 1);
    assert_eq!(fs::read(&fixture.path).expect("stored bytes"), bytes);
}

#[test]
fn complete_scan_marks_unseen_records_stale() {
    let fixture = StoreFixture::new();
    fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    let next = snapshot(
        "snapshot-2",
        "scan-2",
        "complete",
        vec![observed_set(
            "set-a2",
            &fixture.scan_root,
            "Project A/Main.als",
            100,
        )],
        vec![observed_marker(
            "marker-a2",
            &fixture.scan_root,
            "Project A",
        )],
    );
    let result = fixture.store(next);

    assert_eq!(catalog(&result).live_sets.len(), 2);
    assert_eq!(catalog(&result).metadata.stale_live_set_count, 1);
    assert!(catalog(&result).live_sets.iter().any(|record| {
        record.record.display_name == "Remix"
            && record.freshness_status == "not_observed_in_latest_complete_scan"
    }));
}

#[test]
fn partial_scan_retains_unseen_records_without_missing_claim() {
    let fixture = StoreFixture::new();
    fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    let partial = snapshot(
        "snapshot-2",
        "scan-2",
        "partial",
        vec![observed_set(
            "set-a2",
            &fixture.scan_root,
            "Project A/Main.als",
            101,
        )],
        vec![observed_marker(
            "marker-a2",
            &fixture.scan_root,
            "Project A",
        )],
    );
    let result = fixture.store(partial);

    assert_eq!(catalog(&result).metadata.coverage_status, "partial");
    assert_eq!(catalog(&result).metadata.retained_live_set_count, 1);
    assert_eq!(catalog(&result).metadata.stale_live_set_count, 0);
    assert!(catalog(&result).live_sets.iter().any(|record| {
        record.record.display_name == "Remix"
            && record.freshness_status == "retained_from_prior_scan"
            && record.last_observed_scan_run_id == "scan-1"
    }));
}

#[test]
fn changed_complete_scope_retains_unseen_records_without_stale_claim() {
    let fixture = StoreFixture::new();
    fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    let next = snapshot(
        "snapshot-2",
        "scan-2",
        "complete",
        vec![observed_set(
            "set-a2",
            &fixture.scan_root,
            "Project A/Main.als",
            100,
        )],
        vec![observed_marker(
            "marker-a2",
            &fixture.scan_root,
            "Project A",
        )],
    );
    let result = store_project_catalog(&ProjectCatalogStoreRequest {
        store_path: fixture.path.clone(),
        coverage_scope_id: "different-scope".to_string(),
        snapshot: next,
    });

    assert_eq!(catalog(&result).metadata.coverage_status, "scope_changed");
    assert_eq!(catalog(&result).metadata.retained_live_set_count, 1);
    assert_eq!(catalog(&result).metadata.stale_live_set_count, 0);
}

#[test]
fn partial_scan_does_not_shrink_known_folder_membership() {
    let fixture = StoreFixture::new();
    let complete = snapshot(
        "snapshot-1",
        "scan-1",
        "complete",
        vec![
            observed_set("main", &fixture.scan_root, "Project A/Main.als", 100),
            observed_set(
                "alternate",
                &fixture.scan_root,
                "Project A/Alternate.als",
                90,
            ),
            observed_set("backup", &fixture.scan_root, "Project A/Backup/Old.als", 80),
        ],
        vec![observed_marker("marker", &fixture.scan_root, "Project A")],
    );
    fixture.store(complete);
    let partial = snapshot(
        "snapshot-2",
        "scan-2",
        "partial",
        vec![observed_set(
            "main-2",
            &fixture.scan_root,
            "Project A/Main.als",
            101,
        )],
        vec![observed_marker("marker-2", &fixture.scan_root, "Project A")],
    );
    let result = fixture.store(partial);
    let folder = &catalog(&result).project_folders[0].record;

    assert_eq!(folder.main_set_ids.len(), 2);
    assert_eq!(folder.backup_set_ids.len(), 1);
}

#[test]
fn changed_observation_updates_same_occurrence() {
    let fixture = StoreFixture::new();
    let first = snapshot(
        "snapshot-1",
        "scan-1",
        "complete",
        vec![observed_set("old", &fixture.scan_root, "Main.als", 100)],
        Vec::new(),
    );
    fixture.store(first);
    let changed = snapshot(
        "snapshot-2",
        "scan-2",
        "complete",
        vec![observed_set("new", &fixture.scan_root, "Main.als", 999)],
        Vec::new(),
    );
    let result = fixture.store(changed);

    assert_eq!(catalog(&result).live_sets.len(), 1);
    assert_eq!(catalog(&result).live_sets[0].record.file_size, 999);
    assert_eq!(catalog(&result).metadata.revision, 2);
}

#[test]
fn corrupt_or_unsupported_store_fails_without_overwrite() {
    let fixture = StoreFixture::new();
    fs::write(&fixture.path, b"not-json").expect("corrupt fixture");
    let corrupt_before = fs::read(&fixture.path).expect("corrupt bytes");
    let corrupt = fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    assert_eq!(corrupt.operation_status, "failed");
    assert_eq!(
        fs::read(&fixture.path).expect("corrupt bytes"),
        corrupt_before
    );

    fs::remove_file(&fixture.path).expect("remove corrupt fixture");
    fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.path).expect("stored bytes"))
            .expect("valid fixture JSON");
    value["metadata"]["storage_schema_version"] = serde_json::json!("99.0");
    fs::write(
        &fixture.path,
        serde_json::to_vec_pretty(&value).expect("fixture JSON"),
    )
    .expect("unsupported fixture");
    let unsupported_before = fs::read(&fixture.path).expect("unsupported bytes");
    let unsupported = fixture.store(two_projects(&fixture, "snapshot-2", "scan-2", "complete"));
    assert_eq!(unsupported.operation_status, "failed");
    assert_eq!(
        fs::read(&fixture.path).expect("unsupported bytes"),
        unsupported_before
    );
}

#[test]
fn failed_snapshot_is_rejected_without_write() {
    let fixture = StoreFixture::new();
    let mut failed = two_projects(&fixture, "snapshot-1", "scan-1", "complete");
    failed.metadata.build_status = "failed".to_string();
    failed.metadata.error_count = 1;
    failed.errors.push(rescue_catalog::ProjectCatalogError {
        error_code: "FIXTURE_FAILED".to_string(),
        message: "fixture failure".to_string(),
    });
    let result = fixture.store(failed);

    assert_eq!(result.operation_status, "failed");
    assert!(!fixture.path.exists());
}

#[test]
fn atomic_write_leaves_no_owned_temporary_files() {
    let fixture = StoreFixture::new();
    fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    fixture.store(two_projects(&fixture, "snapshot-2", "scan-2", "complete"));
    let names: Vec<String> = fs::read_dir(fixture.path.parent().expect("store parent"))
        .expect("store parent entries")
        .map(|entry| {
            entry
                .expect("store entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    assert_eq!(names, vec!["catalog-state.json"]);
}

#[test]
fn stored_bytes_are_deterministic() {
    let first = StoreFixture::new();
    let mut second = StoreFixture::new();
    second.scan_root = first.scan_root.clone();
    first.store(two_projects(&first, "snapshot-1", "scan-1", "complete"));
    second.store(two_projects(&second, "snapshot-1", "scan-1", "complete"));

    assert_eq!(
        fs::read(&first.path).expect("first bytes"),
        fs::read(&second.path).expect("second bytes")
    );
}

#[test]
fn store_diagnostics_contain_no_private_paths() {
    let fixture = StoreFixture::new();
    fs::write(&fixture.path, b"private corrupt bytes").expect("corrupt store");
    let result = fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    let diagnostics = format!("{:?}", result.errors);

    assert!(!diagnostics.contains(fixture.path.to_string_lossy().as_ref()));
    assert!(!diagnostics.contains(fixture.scan_root.to_string_lossy().as_ref()));
}

#[cfg(unix)]
#[test]
fn store_target_symlink_is_rejected_without_touching_target() {
    use std::os::unix::fs::symlink;

    let fixture = StoreFixture::new();
    let protected = fixture
        .path
        .parent()
        .expect("store parent")
        .join("protected.json");
    fs::write(&protected, b"protected").expect("protected fixture");
    symlink(&protected, &fixture.path).expect("store symlink");

    let result = fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));

    assert_eq!(result.operation_status, "failed");
    assert_eq!(fs::read(&protected).expect("protected bytes"), b"protected");
    assert!(fs::symlink_metadata(&fixture.path)
        .expect("store symlink metadata")
        .file_type()
        .is_symlink());
}

#[test]
fn fake_application_service_uses_stored_catalog_contract() {
    fn visible_rows(catalog: &StoredProjectCatalog) -> Vec<(&str, &str)> {
        catalog
            .live_sets
            .iter()
            .map(|stored| {
                (
                    stored.record.display_name.as_str(),
                    stored.freshness_status.as_str(),
                )
            })
            .collect()
    }

    let fixture = StoreFixture::new();
    let result = fixture.store(two_projects(&fixture, "snapshot-1", "scan-1", "complete"));
    let rows = visible_rows(catalog(&result));

    assert_eq!(rows.len(), 2);
    assert!(rows
        .iter()
        .all(|(_, freshness)| *freshness == "observed_in_latest_scan"));
}
