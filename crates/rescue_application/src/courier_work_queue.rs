use crate::ProjectSelection;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const COURIER_QUEUE_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierWorkItem {
    pub work_item_id: String,
    pub selection: ProjectSelection,
    pub source_display_name: String,
    pub accepted_sequence: u64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierQueueSnapshot {
    pub schema_version: String,
    pub items: Vec<CourierWorkItem>,
    pub pending_item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierQueueError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct CourierWorkQueue {
    items: Vec<CourierWorkItem>,
    next_sequence: u64,
}

impl CourierWorkQueue {
    pub fn add(
        &mut self,
        selection: ProjectSelection,
        source_display_name: String,
    ) -> Result<CourierWorkItem, CourierQueueError> {
        validate_selection(&selection, &source_display_name)?;
        let candidate_key = path_key(&selection.native_als_path).ok_or_else(invalid_selection)?;
        if self.items.iter().any(|item| {
            item.status != "removed"
                && path_key(&item.selection.native_als_path).as_ref() == Some(&candidate_key)
        }) {
            return Err(error(
                "COURIER_DUPLICATE_SOURCE",
                "add",
                "This ALS is already part of the active collection.",
            ));
        }
        self.next_sequence = self.next_sequence.saturating_add(1);
        let item = CourierWorkItem {
            work_item_id: format!("courier-item-{:06}", self.next_sequence),
            selection,
            source_display_name,
            accepted_sequence: self.next_sequence,
            status: "queued".to_string(),
        };
        self.items.push(item.clone());
        Ok(item)
    }

    pub fn remove(&mut self, work_item_id: &str) -> Result<(), CourierQueueError> {
        let item = self.item_mut(work_item_id)?;
        if item.status != "queued" {
            return Err(error(
                "COURIER_ITEM_NOT_REMOVABLE",
                "remove",
                "Only a queued ALS can be removed from the collection.",
            ));
        }
        item.status = "removed".to_string();
        Ok(())
    }

    pub fn mark_processing(&mut self, work_item_ids: &[String]) -> Result<(), CourierQueueError> {
        for work_item_id in work_item_ids {
            if self.item(work_item_id)?.status != "queued" {
                return Err(invalid_state("mark_processing"));
            }
        }
        for work_item_id in work_item_ids {
            self.item_mut(work_item_id)?.status = "processing".to_string();
        }
        Ok(())
    }

    pub fn finish_item(
        &mut self,
        work_item_id: &str,
        outcome: &str,
    ) -> Result<(), CourierQueueError> {
        if !matches!(
            outcome,
            "completed" | "completed_incomplete" | "blocked" | "failed"
        ) {
            return Err(invalid_state("finish_item"));
        }
        let item = self.item_mut(work_item_id)?;
        if item.status != "processing" {
            return Err(invalid_state("finish_item"));
        }
        item.status = outcome.to_string();
        Ok(())
    }

    pub fn queued_items(&self) -> Vec<CourierWorkItem> {
        self.items
            .iter()
            .filter(|item| item.status == "queued")
            .cloned()
            .collect()
    }

    pub fn snapshot(&self) -> CourierQueueSnapshot {
        CourierQueueSnapshot {
            schema_version: COURIER_QUEUE_SCHEMA_VERSION.to_string(),
            items: self.items.clone(),
            pending_item_count: self
                .items
                .iter()
                .filter(|item| item.status == "queued")
                .count(),
        }
    }

    fn item(&self, work_item_id: &str) -> Result<&CourierWorkItem, CourierQueueError> {
        self.items
            .iter()
            .find(|item| item.work_item_id == work_item_id)
            .ok_or_else(|| unknown_item("lookup"))
    }

    fn item_mut(&mut self, work_item_id: &str) -> Result<&mut CourierWorkItem, CourierQueueError> {
        self.items
            .iter_mut()
            .find(|item| item.work_item_id == work_item_id)
            .ok_or_else(|| unknown_item("update"))
    }
}

fn validate_selection(
    selection: &ProjectSelection,
    source_display_name: &str,
) -> Result<(), CourierQueueError> {
    let path = &selection.native_als_path;
    if selection.selection_id.trim().is_empty()
        || source_display_name.trim().is_empty()
        || !path.is_absolute()
        || path.to_str().is_none()
        || !is_als(path)
    {
        return Err(invalid_selection());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| {
        error(
            "COURIER_SOURCE_NOT_REGULAR_ALS",
            "add",
            "The selected ALS is not an available regular file.",
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(error(
            "COURIER_SOURCE_IS_SYMLINK",
            "add",
            "Symbolic-link ALS inputs are not supported.",
        ));
    }
    if !metadata.is_file() {
        return Err(error(
            "COURIER_SOURCE_NOT_REGULAR_ALS",
            "add",
            "The selected ALS is not an available regular file.",
        ));
    }
    Ok(())
}

fn is_als(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("als"))
}

fn path_key(path: &Path) -> Option<String> {
    let value = path.to_str()?;
    if cfg!(windows) {
        Some(value.to_lowercase())
    } else {
        Some(value.to_string())
    }
}

fn invalid_selection() -> CourierQueueError {
    error(
        "COURIER_SELECTION_INVALID",
        "add",
        "Courier intake requires one explicit valid ALS selection.",
    )
}

fn invalid_state(stage: &str) -> CourierQueueError {
    error(
        "COURIER_QUEUE_STATE_INVALID",
        stage,
        "The requested queue transition is not allowed.",
    )
}

fn unknown_item(stage: &str) -> CourierQueueError {
    error(
        "COURIER_ITEM_UNKNOWN",
        stage,
        "The requested courier work item does not exist.",
    )
}

fn error(code: &str, stage: &str, message: &str) -> CourierQueueError {
    CourierQueueError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_key_is_stable_for_native_path() {
        let path = std::path::PathBuf::from("/tmp/Song.als");
        let expected = if cfg!(windows) {
            "/tmp/song.als"
        } else {
            "/tmp/Song.als"
        };
        assert_eq!(path_key(&path).as_deref(), Some(expected));
    }
}
