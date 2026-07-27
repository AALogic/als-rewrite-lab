use rescue_catalog::{scan_assets, AssetInventoryRequest, AssetInventoryResult};
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
            "rescue_inventory_{name}_{}_{stamp}_{id}",
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

    fn scan(&self, max_entries: usize) -> AssetInventoryResult {
        scan_assets(&AssetInventoryRequest {
            scan_run_id: "fixture-run".to_string(),
            roots: vec![self.root.clone()],
            max_entries,
        })
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn scan_roots(roots: Vec<PathBuf>, max_entries: usize) -> AssetInventoryResult {
    scan_assets(&AssetInventoryRequest {
        scan_run_id: "fixture-run".to_string(),
        roots,
        max_entries,
    })
}

#[test]
fn recognized_audio_files_are_hashed() {
    let tree = TempTree::new("recognized");
    tree.file("audio/Sample.WAV", b"abc");
    let result = tree.scan(100);

    assert_eq!(result.metadata.scan_status, "complete");
    assert_eq!(result.file_occurrences.len(), 1);
    assert_eq!(result.file_occurrences[0].extension, "wav");
    assert_eq!(
        result.content_records[0].digest,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn non_audio_files_are_ignored() {
    let tree = TempTree::new("ignored");
    tree.file("notes.txt", b"not audio");
    tree.file("cover.png", b"image");
    let result = tree.scan(100);

    assert!(result.file_occurrences.is_empty());
    assert!(result.content_records.is_empty());
}

#[test]
fn identical_content_keeps_distinct_occurrences() {
    let tree = TempTree::new("same_content");
    tree.file("A/one.wav", b"same");
    tree.file("B/two.aif", b"same");
    let result = tree.scan(100);

    assert_eq!(result.file_occurrences.len(), 2);
    assert_eq!(result.content_records.len(), 1);
    assert_eq!(result.content_records[0].occurrence_ids.len(), 2);
}

#[test]
fn same_name_different_content_stays_distinct() {
    let tree = TempTree::new("same_name");
    tree.file("A/kick.wav", b"first");
    tree.file("B/kick.wav", b"second");
    let result = tree.scan(100);

    assert_eq!(result.file_occurrences.len(), 2);
    assert_eq!(result.content_records.len(), 2);
    assert_ne!(
        result.file_occurrences[0].content_id,
        result.file_occurrences[1].content_id
    );
}

#[cfg(unix)]
#[test]
fn symlinks_are_not_followed() {
    use std::os::unix::fs::symlink;

    let tree = TempTree::new("symlink");
    let external = tree.file("external.wav", b"outside");
    let directory = tree.root.join("real-dir");
    fs::create_dir_all(&directory).expect("real directory should exist");
    tree.file("real-dir/inside.wav", b"inside");
    symlink(&external, tree.root.join("linked.wav")).expect("file symlink should exist");
    symlink(&directory, tree.root.join("linked-dir")).expect("directory symlink should exist");
    let result = tree.scan(100);

    assert_eq!(result.metadata.skipped_symlink_count, 2);
    assert_eq!(result.file_occurrences.len(), 2);
    assert!(result
        .warnings
        .iter()
        .all(|warning| warning.warning_code == "INVENTORY_SYMLINK_SKIPPED"));
}

#[test]
fn invalid_roots_fail_closed() {
    let tree = TempTree::new("invalid");
    let file = tree.file("not-a-directory.wav", b"data");
    let missing = tree.root.join("missing");
    let result = scan_roots(vec![PathBuf::from("relative"), missing, file], 100);

    assert_eq!(result.metadata.scan_status, "failed");
    assert!(result.file_occurrences.is_empty());
    assert_eq!(result.errors.len(), 3);
}

#[test]
fn entry_limit_marks_scan_partial() {
    let tree = TempTree::new("limit");
    tree.file("one.wav", b"one");
    tree.file("two.wav", b"two");
    let result = tree.scan(1);

    assert_eq!(result.metadata.scan_status, "partial");
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "INVENTORY_ENTRY_LIMIT_REACHED"));
}

#[test]
fn overlapping_roots_do_not_duplicate_paths() {
    let tree = TempTree::new("overlap");
    tree.file("nested/one.wav", b"one");
    let result = scan_roots(vec![tree.root.clone(), tree.root.join("nested")], 100);

    assert_eq!(result.file_occurrences.len(), 1);
}

#[test]
fn inventory_is_read_only() {
    let tree = TempTree::new("read_only");
    let path = tree.file("one.wav", b"unchanged");
    let before = fs::read(&path).expect("fixture should be readable");
    let result = tree.scan(100);
    let after = fs::read(&path).expect("fixture should remain readable");

    assert!(result.errors.is_empty());
    assert_eq!(before, after);
}

#[test]
fn inventory_output_is_deterministic() {
    let tree = TempTree::new("deterministic");
    tree.file("B/two.wav", b"two");
    tree.file("A/one.wav", b"one");

    assert_eq!(tree.scan(100), tree.scan(100));
}

#[test]
fn fake_resolution_consumer_uses_inventory_contract() {
    fn content_paths(result: &AssetInventoryResult) -> Vec<(&str, &Path)> {
        result
            .file_occurrences
            .iter()
            .map(|item| (item.content_id.as_str(), item.native_path.as_path()))
            .collect()
    }

    let tree = TempTree::new("consumer");
    tree.file("one.wav", b"one");
    let result = tree.scan(100);
    let records = content_paths(&result);

    assert_eq!(records.len(), 1);
    assert!(records[0].0.starts_with("sha256:"));
    assert!(records[0].1.ends_with("one.wav"));
}
