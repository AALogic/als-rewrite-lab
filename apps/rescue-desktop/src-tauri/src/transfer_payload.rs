use rescue_application::DeliveryCollectionSnapshot;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TransferPayloadError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PayloadCandidate {
    collection_id: String,
    collection_revision: u64,
    target_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttemptPhase {
    Armed,
    #[cfg(any(target_os = "macos", test))]
    Dragging,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveAttempt {
    attempt: TransferPayloadAttempt,
    phase: AttemptPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransferPayloadAttempt {
    pub attempt_id: String,
    pub collection_id: String,
    pub collection_revision: u64,
    pub target_roots: Vec<PathBuf>,
}

#[derive(Default)]
struct TransferPayloadStore {
    candidate: Option<PayloadCandidate>,
    attempt: Option<ActiveAttempt>,
    next_sequence: u64,
}

#[derive(Default)]
pub(crate) struct TransferPayloadState {
    store: Mutex<TransferPayloadStore>,
}

impl TransferPayloadState {
    pub(crate) fn reset(&self) -> Option<String> {
        let mut store = self.store.lock().ok()?;
        store.candidate = None;
        store
            .attempt
            .take()
            .map(|attempt| attempt.attempt.attempt_id)
    }

    pub(crate) fn clear_attempt(&self) -> Result<Option<String>, TransferPayloadError> {
        let mut store = self.lock_store("reset_attempt")?;
        Ok(store
            .attempt
            .take()
            .map(|attempt| attempt.attempt.attempt_id))
    }

    pub(crate) fn register_collection(&self, snapshot: &DeliveryCollectionSnapshot) -> bool {
        let target_roots = snapshot
            .items
            .iter()
            .map(|item| item.target_project_root.clone())
            .collect::<Vec<_>>();
        let candidate = (!snapshot.collection_id.trim().is_empty()
            && snapshot.revision > 0
            && !target_roots.is_empty())
        .then(|| PayloadCandidate {
            collection_id: snapshot.collection_id.clone(),
            collection_revision: snapshot.revision,
            target_roots,
        });
        if let Ok(mut store) = self.store.lock() {
            store.attempt = None;
            store.candidate = candidate;
            return store.candidate.is_some();
        }
        false
    }

    pub(crate) fn prepare_attempt(
        &self,
        collection_id: &str,
        collection_revision: u64,
    ) -> Result<TransferPayloadAttempt, TransferPayloadError> {
        let mut store = self.lock_store("prepare_attempt")?;
        if store.attempt.is_some() {
            return Err(payload_error(
                "PAYLOAD_ATTEMPT_BUSY",
                "prepare_attempt",
                "Another payload attempt is already active.",
            ));
        }
        let candidate = store.candidate.clone().ok_or_else(|| {
            payload_error(
                "PAYLOAD_CANDIDATE_MISSING",
                "prepare_attempt",
                "No ready collection is available as a payload.",
            )
        })?;
        if collection_id != candidate.collection_id
            || collection_revision != candidate.collection_revision
        {
            return Err(payload_error(
                "PAYLOAD_CANDIDATE_MISMATCH",
                "prepare_attempt",
                "The requested collection revision does not match the available payload.",
            ));
        }
        validate_targets(&candidate.target_roots)?;
        store.next_sequence = store.next_sequence.saturating_add(1);
        let attempt = TransferPayloadAttempt {
            attempt_id: format!(
                "payload-attempt-{}-{}",
                std::process::id(),
                store.next_sequence
            ),
            collection_id: candidate.collection_id,
            collection_revision: candidate.collection_revision,
            target_roots: candidate.target_roots,
        };
        store.attempt = Some(ActiveAttempt {
            attempt: attempt.clone(),
            phase: AttemptPhase::Armed,
        });
        Ok(attempt)
    }

    #[cfg(any(target_os = "macos", test))]
    pub(crate) fn begin_attempt(
        &self,
        attempt_id: &str,
    ) -> Result<TransferPayloadAttempt, TransferPayloadError> {
        let mut store = self.lock_store("begin_attempt")?;
        let attempt = store
            .attempt
            .as_mut()
            .ok_or_else(|| stale_error("begin_attempt"))?;
        if attempt.attempt.attempt_id != attempt_id || attempt.phase != AttemptPhase::Armed {
            return Err(stale_error("begin_attempt"));
        }
        attempt.phase = AttemptPhase::Dragging;
        Ok(attempt.attempt.clone())
    }

    #[cfg(any(target_os = "macos", test))]
    pub(crate) fn finish_attempt(
        &self,
        attempt_id: &str,
    ) -> Result<TransferPayloadAttempt, TransferPayloadError> {
        let mut store = self.lock_store("finish_attempt")?;
        let attempt = store
            .attempt
            .as_ref()
            .ok_or_else(|| stale_error("finish_attempt"))?;
        if attempt.attempt.attempt_id != attempt_id {
            return Err(stale_error("finish_attempt"));
        }
        let finished = attempt.attempt.clone();
        store.attempt = None;
        Ok(finished)
    }

    pub(crate) fn cancel_armed(&self, attempt_id: &str) -> Result<(), TransferPayloadError> {
        let mut store = self.lock_store("cancel_attempt")?;
        let attempt = store
            .attempt
            .as_ref()
            .ok_or_else(|| stale_error("cancel_attempt"))?;
        if attempt.attempt.attempt_id != attempt_id || attempt.phase != AttemptPhase::Armed {
            return Err(stale_error("cancel_attempt"));
        }
        store.attempt = None;
        Ok(())
    }

    fn lock_store(
        &self,
        stage: &str,
    ) -> Result<MutexGuard<'_, TransferPayloadStore>, TransferPayloadError> {
        self.store.lock().map_err(|_| {
            payload_error(
                "PAYLOAD_STATE_UNAVAILABLE",
                stage,
                "The transfer payload state is unavailable.",
            )
        })
    }
}

fn validate_targets(paths: &[PathBuf]) -> Result<(), TransferPayloadError> {
    if paths.is_empty() {
        return Err(payload_error(
            "PAYLOAD_CANDIDATE_INVALID",
            "prepare_attempt",
            "The collection payload is empty.",
        ));
    }
    let mut seen = HashSet::new();
    for path in paths {
        let key = path_key(path).ok_or_else(|| invalid_target("PAYLOAD_CANDIDATE_INVALID"))?;
        if !seen.insert(key) {
            return Err(invalid_target("PAYLOAD_CANDIDATE_INVALID"));
        }
        validate_target(path)?;
    }
    Ok(())
}

fn validate_target(path: &Path) -> Result<(), TransferPayloadError> {
    if !path.is_absolute() {
        return Err(invalid_target("PAYLOAD_CANDIDATE_INVALID"));
    }
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| invalid_target("PAYLOAD_TARGET_MISSING"))?;
    if metadata.file_type().is_symlink() {
        return Err(invalid_target("PAYLOAD_TARGET_IS_SYMLINK"));
    }
    if !metadata.is_dir() {
        return Err(invalid_target("PAYLOAD_TARGET_NOT_DIRECTORY"));
    }
    Ok(())
}

fn path_key(path: &Path) -> Option<String> {
    let value = path.to_str()?;
    if cfg!(windows) {
        Some(value.to_lowercase())
    } else {
        Some(value.to_string())
    }
}

fn invalid_target(code: &str) -> TransferPayloadError {
    payload_error(
        code,
        "prepare_attempt",
        "A collection payload directory is unavailable or unsupported.",
    )
}

fn stale_error(stage: &str) -> TransferPayloadError {
    payload_error(
        "PAYLOAD_ATTEMPT_STALE",
        stage,
        "The payload attempt is stale or already used.",
    )
}

fn payload_error(code: &str, stage: &str, message: &str) -> TransferPayloadError {
    TransferPayloadError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
#[path = "transfer_payload_tests.rs"]
mod tests;
