use rescue_analyzer::{discover_project, ProjectDiscoveryRequest};
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
            .expect("clock should be after Unix epoch")
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rescue_project_discovery_{name}_{}_{stamp}_{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root should exist");
        Self { root }
    }

    fn marker(&self, relative_root: &str) -> PathBuf {
        let root = self.root.join(relative_root);
        fs::create_dir_all(root.join("Ableton Project Info")).expect("marker should exist");
        root
    }

    fn als(&self, relative: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("ALS should have parent"))
            .expect("ALS parent should exist");
        fs::write(&path, b"fixture").expect("ALS fixture should exist");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn discover(path: &Path) -> rescue_analyzer::ProjectDiscoveryResult {
    discover_project(&ProjectDiscoveryRequest {
        source_als_path: path.to_path_buf(),
    })
}

#[test]
fn exact_marker_confirms_project_root() {
    let tree = TempTree::new("exact");
    let project = tree.marker("Song Project");
    let als = tree.als("Song Project/Song.als");
    let result = discover(&als);

    assert_eq!(result.discovery_status, "confirmed");
    assert_eq!(
        result.confirmed_project_root.as_deref(),
        Some(project.as_path())
    );
    assert_eq!(result.set_location, "project_root");
}

#[test]
fn nested_set_uses_marker_bearing_ancestor() {
    let tree = TempTree::new("nested");
    let project = tree.marker("Song Project");
    let als = tree.als("Song Project/Sets/Versions/Song.als");
    let result = discover(&als);

    assert_eq!(
        result.confirmed_project_root.as_deref(),
        Some(project.as_path())
    );
    assert_eq!(result.set_location, "project_subdirectory");
    assert!(result.candidates[0].depth_from_set > 0);
}

#[test]
fn backup_set_is_labeled_without_main_set_claim() {
    let tree = TempTree::new("backup");
    tree.marker("Song Project");
    let als = tree.als("Song Project/Backup/Song [date].als");
    let result = discover(&als);
    let json = serde_json::to_string(&result).expect("result should serialize");

    assert_eq!(result.discovery_status, "confirmed");
    assert_eq!(result.set_location, "backup_candidate");
    assert!(!json.contains("main_set"));
}

#[test]
fn absent_marker_keeps_project_root_unknown() {
    let tree = TempTree::new("absent");
    let als = tree.als("Loose/Set.als");
    let result = discover(&als);

    assert_eq!(result.discovery_status, "unknown");
    assert_eq!(result.confirmed_project_root, None);
    assert_ne!(result.confirmed_project_root.as_deref(), als.parent());
}

#[test]
fn nested_markers_are_ambiguous() {
    let tree = TempTree::new("ambiguous");
    tree.marker("Outer");
    tree.marker("Outer/Inner");
    let als = tree.als("Outer/Inner/Set.als");
    let result = discover(&als);

    assert_eq!(result.candidates.len(), 2);
    assert_eq!(result.discovery_status, "ambiguous");
    assert_eq!(result.confirmed_project_root, None);
}

#[cfg(unix)]
#[test]
fn marker_symlink_is_not_confirmed() {
    use std::os::unix::fs::symlink;

    let tree = TempTree::new("marker_symlink");
    let project = tree.root.join("Project");
    let marker_target = tree.root.join("marker-target");
    fs::create_dir_all(&project).expect("project should exist");
    fs::create_dir_all(&marker_target).expect("target should exist");
    symlink(&marker_target, project.join("Ableton Project Info"))
        .expect("marker symlink should exist");
    let als = tree.als("Project/Set.als");
    let result = discover(&als);

    assert_eq!(result.discovery_status, "unknown");
    assert_eq!(result.confirmed_project_root, None);
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "PROJECT_MARKER_IS_SYMLINK"));
}

#[cfg(unix)]
#[test]
fn source_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let tree = TempTree::new("source_symlink");
    let target = tree.als("Project/Real.als");
    let link = tree.root.join("Project/Link.als");
    symlink(target, &link).expect("source symlink should exist");
    let result = discover(&link);

    assert_eq!(result.discovery_status, "unsupported");
    assert_eq!(
        result.errors[0].error_code,
        "PROJECT_DISCOVERY_SOURCE_IS_SYMLINK"
    );
}

#[test]
fn discovery_is_read_only() {
    let tree = TempTree::new("read_only");
    tree.marker("Project");
    let als = tree.als("Project/Set.als");
    let before = fs::read(&als).expect("fixture should be readable");
    let result = discover(&als);
    let after = fs::read(&als).expect("fixture should remain readable");

    assert!(result.errors.is_empty());
    assert_eq!(before, after);
}
