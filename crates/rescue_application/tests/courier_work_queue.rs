use rescue_application::{CourierWorkQueue, ProjectSelection};
use std::path::{Path, PathBuf};

fn fixture() -> tempfile::TempDir {
    tempfile::tempdir().expect("queue fixture")
}

fn write_als(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, bytes).expect("write ALS fixture");
    path
}

fn selection(path: PathBuf, id: &str) -> ProjectSelection {
    ProjectSelection {
        selection_id: id.to_string(),
        selection_source: "manual".to_string(),
        live_set_id: None,
        native_als_path: path,
        catalog_revision: None,
        observation_fingerprint: None,
        freshness_status: "manual_unverified".to_string(),
    }
}

#[test]
fn courier_queue_preserves_acceptance_order() {
    let temp = fixture();
    let first = write_als(temp.path(), "A.als", b"a");
    let second = write_als(temp.path(), "B.als", b"b");
    let mut queue = CourierWorkQueue::default();
    queue
        .add(selection(first, "a"), "A.als".to_string())
        .expect("first");
    queue
        .add(selection(second, "b"), "B.als".to_string())
        .expect("second");
    let snapshot = queue.snapshot();
    assert_eq!(snapshot.pending_item_count, 2);
    assert_eq!(snapshot.items[0].source_display_name, "A.als");
    assert_eq!(snapshot.items[1].source_display_name, "B.als");
    assert!(snapshot.items[0].accepted_sequence < snapshot.items[1].accepted_sequence);
}

#[test]
fn courier_queue_rejects_active_duplicate_source() {
    let temp = fixture();
    let path = write_als(temp.path(), "A.als", b"a");
    let mut queue = CourierWorkQueue::default();
    queue
        .add(selection(path.clone(), "a"), "A.als".to_string())
        .expect("first");
    let failure = queue
        .add(selection(path, "b"), "A.als".to_string())
        .expect_err("duplicate");
    assert_eq!(failure.error_code, "COURIER_DUPLICATE_SOURCE");
}

#[test]
fn removed_item_can_be_added_again() {
    let temp = fixture();
    let path = write_als(temp.path(), "A.als", b"a");
    let mut queue = CourierWorkQueue::default();
    let first = queue
        .add(selection(path.clone(), "a"), "A.als".to_string())
        .expect("first");
    queue.remove(&first.work_item_id).expect("remove");
    let second = queue
        .add(selection(path, "b"), "A.als".to_string())
        .expect("re-add");
    assert_ne!(first.work_item_id, second.work_item_id);
}

#[test]
fn processing_item_cannot_be_removed() {
    let temp = fixture();
    let path = write_als(temp.path(), "A.als", b"a");
    let mut queue = CourierWorkQueue::default();
    let item = queue
        .add(selection(path, "a"), "A.als".to_string())
        .expect("add");
    queue
        .mark_processing(std::slice::from_ref(&item.work_item_id))
        .expect("processing");
    let failure = queue.remove(&item.work_item_id).expect_err("must block");
    assert_eq!(failure.error_code, "COURIER_ITEM_NOT_REMOVABLE");
}

#[test]
fn non_als_directory_and_symlink_are_rejected() {
    let temp = fixture();
    let text = temp.path().join("note.txt");
    std::fs::write(&text, b"text").expect("text fixture");
    let directory = temp.path().join("Folder.als");
    std::fs::create_dir(&directory).expect("directory fixture");
    let mut queue = CourierWorkQueue::default();
    assert!(queue
        .add(selection(text, "text"), "note.txt".to_string())
        .is_err());
    assert!(queue
        .add(selection(directory, "dir"), "Folder.als".to_string())
        .is_err());
    #[cfg(unix)]
    {
        let real = write_als(temp.path(), "Real.als", b"real");
        let link = temp.path().join("Link.als");
        std::os::unix::fs::symlink(real, &link).expect("symlink fixture");
        let failure = queue
            .add(selection(link, "link"), "Link.als".to_string())
            .expect_err("link");
        assert_eq!(failure.error_code, "COURIER_SOURCE_IS_SYMLINK");
    }
}

#[test]
fn queue_errors_are_path_free() {
    let secret = PathBuf::from("/Users/private/Secret.als");
    let mut queue = CourierWorkQueue::default();
    let failure = queue
        .add(
            selection(secret.clone(), "secret"),
            "Secret.als".to_string(),
        )
        .expect_err("missing");
    let json = serde_json::to_string(&failure).expect("serialize");
    assert!(!json.contains(&secret.to_string_lossy().to_string()));
    assert!(!json.contains("Secret.als"));
}

#[test]
fn queue_never_mutates_source_files() {
    let temp = fixture();
    let path = write_als(temp.path(), "A.als", b"original bytes");
    let before = std::fs::read(&path).expect("before");
    let mut queue = CourierWorkQueue::default();
    let item = queue
        .add(selection(path.clone(), "a"), "A.als".to_string())
        .expect("add");
    queue.remove(&item.work_item_id).expect("remove");
    assert_eq!(std::fs::read(path).expect("after"), before);
}
