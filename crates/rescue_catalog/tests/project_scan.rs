use rescue_catalog::{
    scan_projects, scan_projects_controlled, ProjectScanObserver, ProjectScanProgress,
    ProjectScanRequest, ProjectScanResult,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rescue_project_scan_{name}_{}_{stamp}_{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root should exist");
        Self { root }
    }

    fn file(&self, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture file should have parent"))
            .expect("fixture parent should exist");
        fs::write(&path, bytes).expect("fixture file should be written");
        path
    }

    fn directory(&self, relative: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(&path).expect("fixture directory should exist");
        path
    }

    fn request(&self) -> ProjectScanRequest {
        request(vec![self.root.clone()])
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn request(roots: Vec<PathBuf>) -> ProjectScanRequest {
    ProjectScanRequest {
        scan_run_id: "project-scan-fixture".to_string(),
        roots,
        excluded_roots: Vec::new(),
        max_entries: 10_000,
        max_depth: None,
        traversal_policy_version: "project_scan_v0.1".to_string(),
    }
}

fn paths(result: &ProjectScanResult) -> Vec<PathBuf> {
    result
        .als_files
        .iter()
        .map(|item| item.native_path.clone())
        .collect()
}

#[test]
fn standard_project_entries_are_observed() {
    let tree = TempTree::new("standard");
    let als = tree.file("Standard Project/Main.als", b"not parsed");
    let marker = tree.directory("Standard Project/Ableton Project Info");

    let result = scan_projects(&tree.request());

    assert_eq!(result.metadata.scan_status, "complete");
    assert_eq!(paths(&result), vec![als]);
    assert_eq!(result.project_markers.len(), 1);
    assert_eq!(result.project_markers[0].marker_path, marker);
    assert!(result.project_markers[0]
        .project_root_candidate
        .ends_with("Standard Project"));
}

#[test]
fn backup_and_standalone_als_are_retained() {
    let tree = TempTree::new("backup-standalone");
    let backup = tree.file("Project/Backup/Main [2026-08-01 010101].als", b"backup");
    let standalone = tree.file("Standalone/orphan.als", b"orphan");

    let result = scan_projects(&tree.request());
    let observed = paths(&result);

    assert_eq!(observed.len(), 2);
    assert!(observed.contains(&backup));
    assert!(observed.contains(&standalone));
}

#[test]
fn invalid_gzip_als_is_not_opened() {
    let tree = TempTree::new("invalid-gzip");
    let path = tree.file("Broken/not-gzip.als", b"definitely not gzip");

    let result = scan_projects(&tree.request());

    assert_eq!(result.metadata.scan_status, "complete");
    assert_eq!(paths(&result), vec![path]);
    assert!(result.errors.is_empty());
}

#[test]
fn uppercase_als_is_recognized() {
    let tree = TempTree::new("uppercase");
    let path = tree.file("Project/LOUD.ALS", b"candidate");

    let result = scan_projects(&tree.request());

    assert_eq!(paths(&result), vec![path]);
    assert_eq!(result.als_files[0].extension, "als");
}

#[test]
fn non_als_files_are_ignored() {
    let tree = TempTree::new("ignored");
    tree.file("Project/notes.txt", b"notes");
    tree.file("Project/audio.wav", b"audio");
    tree.file("Project/fake.als.txt", b"not an als");

    let result = scan_projects(&tree.request());

    assert!(result.als_files.is_empty());
}

#[test]
fn multiple_main_sets_are_not_collapsed() {
    let tree = TempTree::new("multi-set");
    let first = tree.file("Project/First.als", b"first");
    let second = tree.file("Project/Second.als", b"second");
    tree.directory("Project/Ableton Project Info");

    let result = scan_projects(&tree.request());

    assert_eq!(paths(&result), vec![first, second]);
    assert_eq!(result.metadata.als_file_count, 2);
}

#[test]
fn same_names_in_different_roots_stay_distinct() {
    let first = TempTree::new("same-name-a");
    let second = TempTree::new("same-name-b");
    let first_path = first.file("Same Project/Main.als", b"one");
    let second_path = second.file("Same Project/Main.als", b"two");

    let result = scan_projects(&request(vec![first.root.clone(), second.root.clone()]));
    let observed = paths(&result);

    assert_eq!(observed.len(), 2);
    assert!(observed.contains(&first_path));
    assert!(observed.contains(&second_path));
}

#[test]
fn overlapping_roots_do_not_duplicate_observations() {
    let tree = TempTree::new("overlapping");
    let als = tree.file("Nested/Main.als", b"main");

    let result = scan_projects(&request(vec![
        tree.root.clone(),
        tree.root.join("Nested"),
        tree.root.clone(),
    ]));

    assert_eq!(paths(&result), vec![als]);
    assert_eq!(result.metadata.accepted_root_count, 1);
}

#[test]
fn explicit_exclusions_are_honored() {
    let tree = TempTree::new("excluded");
    let kept = tree.file("Kept/Main.als", b"kept");
    tree.file("Excluded/Ignored.als", b"ignored");
    let mut request = tree.request();
    request.excluded_roots = vec![tree.root.join("Excluded")];

    let result = scan_projects(&request);

    assert_eq!(paths(&result), vec![kept]);
    assert_eq!(result.metadata.scan_status, "complete");
    assert_eq!(result.metadata.skipped_excluded_count, 1);
}

#[test]
fn symlinks_are_not_followed() {
    let tree = TempTree::new("symlinks");
    let real = tree.file("Real/Main.als", b"real");
    let directory = tree.directory("Real");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&real, tree.root.join("Linked.als")).expect("file symlink should exist");
        symlink(&directory, tree.root.join("Linked Project"))
            .expect("directory symlink should exist");
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::{symlink_dir, symlink_file};
        if symlink_file(&real, tree.root.join("Linked.als")).is_err()
            || symlink_dir(&directory, tree.root.join("Linked Project")).is_err()
        {
            return;
        }
    }

    let result = scan_projects(&tree.request());

    assert_eq!(paths(&result), vec![real]);
    assert_eq!(result.metadata.skipped_symlink_count, 2);
    assert!(result
        .warnings
        .iter()
        .all(|warning| warning.warning_code == "PROJECT_SCAN_SYMLINK_SKIPPED"));
}

#[test]
fn entry_limit_marks_project_scan_partial() {
    let tree = TempTree::new("entry-limit");
    tree.file("A.als", b"a");
    tree.file("B.als", b"b");
    let mut request = tree.request();
    request.max_entries = 1;

    let result = scan_projects(&request);

    assert_eq!(result.metadata.scan_status, "partial");
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "PROJECT_SCAN_ENTRY_LIMIT_REACHED"));
}

#[test]
fn depth_limit_marks_project_scan_partial() {
    let tree = TempTree::new("depth-limit");
    let root_set = tree.file("Root.als", b"root");
    tree.file("Nested/Deep.als", b"deep");
    let mut request = tree.request();
    request.max_depth = Some(0);

    let result = scan_projects(&request);

    assert_eq!(paths(&result), vec![root_set]);
    assert_eq!(result.metadata.scan_status, "partial");
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "PROJECT_SCAN_DEPTH_LIMIT_REACHED"));
}

#[test]
fn inaccessible_directory_preserves_partial_results() {
    let tree = TempTree::new("partial-root");
    let observed = tree.file("Main.als", b"main");
    let missing = tree.root.with_file_name(format!(
        "{}-missing",
        tree.root
            .file_name()
            .expect("fixture root should have a name")
            .to_string_lossy()
    ));

    let result = scan_projects(&request(vec![tree.root.clone(), missing]));

    assert_eq!(paths(&result), vec![observed]);
    assert_eq!(result.metadata.scan_status, "partial");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROJECT_SCAN_ROOT_NOT_FOUND"));
}

#[derive(Default)]
struct RecordingObserver {
    events: Vec<ProjectScanProgress>,
    cancel_after_entries: Option<usize>,
}

impl ProjectScanObserver for RecordingObserver {
    fn is_cancelled(&self) -> bool {
        self.cancel_after_entries.is_some_and(|limit| {
            self.events
                .last()
                .is_some_and(|event| event.entries_visited >= limit)
        })
    }

    fn on_progress(&mut self, progress: &ProjectScanProgress) {
        self.events.push(progress.clone());
    }
}

#[test]
fn cancellation_returns_collected_observations() {
    let tree = TempTree::new("cancel");
    tree.file("A/one.als", b"one");
    tree.file("B/two.als", b"two");
    tree.file("C/three.als", b"three");
    let mut observer = RecordingObserver {
        events: Vec::new(),
        cancel_after_entries: Some(1),
    };

    let result = scan_projects_controlled(&tree.request(), &mut observer);

    assert_eq!(result.metadata.scan_status, "cancelled");
    assert!(result.metadata.entries_visited >= 1);
    assert!(result.metadata.entries_visited < 6);
    assert_eq!(
        observer.events.last().map(|event| event.stage.as_str()),
        Some("cancelled")
    );
}

#[test]
fn progress_events_contain_no_private_paths() {
    let tree = TempTree::new("progress-redaction");
    tree.file("Private Project/secret-name.als", b"secret");
    let mut observer = RecordingObserver::default();

    let _ = scan_projects_controlled(&tree.request(), &mut observer);

    assert!(!observer.events.is_empty());
    let debug = format!("{:?}", observer.events);
    assert!(!debug.contains(&tree.root.to_string_lossy().to_string()));
    assert!(!debug.contains("Private Project"));
    assert!(!debug.contains("secret-name.als"));
}

#[test]
fn project_scan_is_read_only() {
    let tree = TempTree::new("read-only");
    let main = tree.file("Project/Main.als", b"unchanged");
    tree.directory("Project/Ableton Project Info");
    let before_entries = relative_entries(&tree.root);
    let before_bytes = fs::read(&main).expect("fixture should be readable");

    let result = scan_projects(&tree.request());

    let after_entries = relative_entries(&tree.root);
    let after_bytes = fs::read(&main).expect("fixture should remain readable");
    assert!(result.errors.is_empty());
    assert_eq!(before_entries, after_entries);
    assert_eq!(before_bytes, after_bytes);
}

#[test]
fn project_scan_output_is_deterministic() {
    let tree = TempTree::new("deterministic");
    tree.file("B/two.als", b"two");
    tree.file("A/one.als", b"one");
    tree.directory("A/Ableton Project Info");

    assert_eq!(
        scan_projects(&tree.request()),
        scan_projects(&tree.request())
    );
}

#[test]
fn fake_catalog_builder_uses_project_scan_contract() {
    fn physical_evidence(result: &ProjectScanResult) -> (Vec<&Path>, Vec<&Path>) {
        let sets = result
            .als_files
            .iter()
            .map(|set| set.native_path.as_path())
            .collect();
        let markers = result
            .project_markers
            .iter()
            .map(|marker| marker.project_root_candidate.as_path())
            .collect();
        (sets, markers)
    }

    let tree = TempTree::new("downstream");
    tree.file("Project/Main.als", b"main");
    tree.directory("Project/Ableton Project Info");
    let result = scan_projects(&tree.request());
    let (sets, markers) = physical_evidence(&result);

    assert_eq!(sets.len(), 1);
    assert_eq!(markers.len(), 1);
}

fn relative_entries(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut children: Vec<_> = fs::read_dir(&directory)
            .expect("fixture should be readable")
            .map(|entry| entry.expect("fixture entry should be readable").path())
            .collect();
        children.sort();
        for child in children {
            let metadata = fs::symlink_metadata(&child).expect("fixture metadata");
            entries.push(
                child
                    .strip_prefix(root)
                    .expect("fixture entry should be inside root")
                    .to_path_buf(),
            );
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                pending.push(child);
            }
        }
    }
    entries.sort();
    entries
}
