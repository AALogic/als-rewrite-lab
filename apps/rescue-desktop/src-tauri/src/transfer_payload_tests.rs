use super::*;
use rescue_application::{CourierCollectionItem, DeliveryCollectionSnapshot};
use std::path::PathBuf;
use tempfile::TempDir;

fn collection(root: &std::path::Path, revision: u64, count: usize) -> DeliveryCollectionSnapshot {
    let items = (0..count)
        .map(|index| {
            let target = root.join(format!("Project-{index} Rescue Project"));
            std::fs::create_dir_all(&target).expect("target fixture");
            CourierCollectionItem {
                work_item_id: format!("item-{index}"),
                source_display_name: format!("Set-{index}.als"),
                copy_result_request_id: format!("copy-{index}"),
                target_project_root: target,
                outcome: "completed".to_string(),
                omitted_asset_count: 0,
            }
        })
        .collect();
    DeliveryCollectionSnapshot {
        schema_version: "0.1".to_string(),
        collection_id: "collection-1".to_string(),
        revision,
        items,
    }
}

fn ready_state() -> (TempDir, TransferPayloadState, DeliveryCollectionSnapshot) {
    let temp = tempfile::tempdir().expect("fixture root");
    let snapshot = collection(temp.path(), 1, 2);
    let state = TransferPayloadState::default();
    assert!(state.register_collection(&snapshot));
    (temp, state, snapshot)
}

#[test]
fn latest_collection_revision_is_the_only_payload_candidate() {
    let temp = tempfile::tempdir().expect("fixture root");
    let state = TransferPayloadState::default();
    assert!(state.register_collection(&collection(temp.path(), 1, 1)));
    assert!(state.register_collection(&collection(temp.path(), 2, 2)));
    let stale = state.prepare_attempt("collection-1", 1);
    let current = state.prepare_attempt("collection-1", 2);
    assert_eq!(
        stale.expect_err("stale").error_code,
        "PAYLOAD_CANDIDATE_MISMATCH"
    );
    assert!(current.is_ok());
}

#[test]
fn multi_directory_attempt_preserves_snapshot_order() {
    let (_temp, state, snapshot) = ready_state();
    let attempt = state.prepare_attempt("collection-1", 1).expect("attempt");
    let expected = snapshot
        .items
        .iter()
        .map(|item| item.target_project_root.clone())
        .collect::<Vec<_>>();
    assert_eq!(attempt.target_roots, expected);
    assert_eq!(attempt.collection_revision, 1);
}

#[test]
fn stale_collection_revision_is_rejected() {
    let (_temp, state, _snapshot) = ready_state();
    let failure = state.prepare_attempt("collection-1", 9).expect_err("stale");
    assert_eq!(failure.error_code, "PAYLOAD_CANDIDATE_MISMATCH");
}

#[test]
fn invalid_member_blocks_entire_drag_attempt() {
    let temp = tempfile::tempdir().expect("fixture root");
    let mut snapshot = collection(temp.path(), 1, 2);
    snapshot.items[1].target_project_root = temp.path().join("missing");
    let state = TransferPayloadState::default();
    assert!(state.register_collection(&snapshot));
    let failure = state
        .prepare_attempt("collection-1", 1)
        .expect_err("invalid member");
    assert_eq!(failure.error_code, "PAYLOAD_TARGET_MISSING");
    #[cfg(unix)]
    {
        let real = temp.path().join("real");
        let link = temp.path().join("link");
        std::fs::create_dir(&real).expect("real");
        std::os::unix::fs::symlink(&real, &link).expect("link");
        snapshot.items[1].target_project_root = link;
        assert!(state.register_collection(&snapshot));
        let failure = state
            .prepare_attempt("collection-1", 1)
            .expect_err("symlink");
        assert_eq!(failure.error_code, "PAYLOAD_TARGET_IS_SYMLINK");
    }
}

#[test]
fn concurrent_payload_attempt_is_rejected() {
    let (_temp, state, _snapshot) = ready_state();
    let first = state.prepare_attempt("collection-1", 1).expect("first");
    let second = state
        .prepare_attempt("collection-1", 1)
        .expect_err("second");
    assert_eq!(second.error_code, "PAYLOAD_ATTEMPT_BUSY");
    state.cancel_armed(&first.attempt_id).expect("cleanup");
}

#[test]
fn payload_attempt_is_one_shot() {
    let (_temp, state, _snapshot) = ready_state();
    let attempt = state.prepare_attempt("collection-1", 1).expect("attempt");
    state.begin_attempt(&attempt.attempt_id).expect("begin");
    assert_eq!(
        state
            .begin_attempt(&attempt.attempt_id)
            .expect_err("repeat")
            .error_code,
        "PAYLOAD_ATTEMPT_STALE"
    );
    state.finish_attempt(&attempt.attempt_id).expect("finish");
}

#[test]
fn terminal_drag_preserves_candidate_for_retry() {
    let (_temp, state, _snapshot) = ready_state();
    let first = state.prepare_attempt("collection-1", 1).expect("first");
    state.begin_attempt(&first.attempt_id).expect("begin");
    state.finish_attempt(&first.attempt_id).expect("finish");
    let retry = state.prepare_attempt("collection-1", 1).expect("retry");
    assert_ne!(first.attempt_id, retry.attempt_id);
}

#[test]
fn reset_after_terminal_attempt_has_no_attempt_identity() {
    let (_temp, state, _snapshot) = ready_state();
    let attempt = state.prepare_attempt("collection-1", 1).expect("attempt");
    state.begin_attempt(&attempt.attempt_id).expect("begin");
    state.finish_attempt(&attempt.attempt_id).expect("finish");
    assert_eq!(state.reset(), None);
}

#[test]
fn explicit_retry_reset_clears_only_the_attempt() {
    let (_temp, state, _snapshot) = ready_state();
    let first = state.prepare_attempt("collection-1", 1).expect("first");
    assert_eq!(
        state.clear_attempt().expect("clear").as_deref(),
        Some(first.attempt_id.as_str())
    );
    let retry = state.prepare_attempt("collection-1", 1).expect("retry");
    assert_ne!(first.attempt_id, retry.attempt_id);
}

#[test]
fn reset_clears_hung_attempt_and_candidate() {
    let (temp, state, _snapshot) = ready_state();
    let first = state.prepare_attempt("collection-1", 1).expect("first");
    state.begin_attempt(&first.attempt_id).expect("begin");
    assert_eq!(state.reset().as_deref(), Some(first.attempt_id.as_str()));
    assert_eq!(
        state
            .prepare_attempt("collection-1", 1)
            .expect_err("cleared")
            .error_code,
        "PAYLOAD_CANDIDATE_MISSING"
    );
    assert!(state.register_collection(&collection(temp.path(), 2, 1)));
}

#[test]
fn payload_errors_are_path_free() {
    let secret = PathBuf::from("/Users/private/Music/Secret Rescue Project");
    let state = TransferPayloadState::default();
    let failure = state.prepare_attempt("collection", 1).expect_err("missing");
    let json = serde_json::to_string(&failure).expect("serialize error");
    assert!(!json.contains(&secret.to_string_lossy().to_string()));
    assert!(!json.contains("target_roots"));
}
