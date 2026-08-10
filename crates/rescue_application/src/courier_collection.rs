use crate::courier_collection_result::{collect_updates, validate_result};
use crate::{
    BatchCopyResult, CourierQueueError, CourierQueueSnapshot, CourierWorkQueue, ProjectSelection,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[path = "courier_collection_handoff.rs"]
mod handoff;
pub use handoff::CourierHandoffSnapshot;
use handoff::CourierHandoffState;

pub const COURIER_COLLECTION_SCHEMA_VERSION: &str = "0.2";
const DELIVERY_COLLECTION_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierBatchWave {
    pub wave_id: String,
    pub collection_id: String,
    pub base_revision: u64,
    pub selections: Vec<ProjectSelection>,
    pub destination_parent: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierCollectionItem {
    pub work_item_id: String,
    pub source_display_name: String,
    pub copy_result_request_id: String,
    pub target_project_root: PathBuf,
    pub outcome: String,
    pub omitted_asset_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierCollectionSnapshot {
    pub schema_version: String,
    pub collection_id: String,
    pub revision: u64,
    pub work_phase: String,
    pub handoff: CourierHandoffSnapshot,
    pub destination_parent: PathBuf,
    pub queue: CourierQueueSnapshot,
    pub items: Vec<CourierCollectionItem>,
    pub completed_item_count: usize,
    pub incomplete_item_count: usize,
    pub failed_item_count: usize,
    pub omitted_asset_count: usize,
    pub last_wave_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryCollectionSnapshot {
    pub schema_version: String,
    pub collection_id: String,
    pub revision: u64,
    pub items: Vec<CourierCollectionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierCollectionError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CourierCollectionOrchestrator {
    collection_id: String,
    revision: u64,
    work_phase: String,
    handoff: CourierHandoffState,
    destination_parent: PathBuf,
    queue: CourierWorkQueue,
    items: Vec<CourierCollectionItem>,
    failed_item_count: usize,
    omitted_asset_count: usize,
    last_wave_id: Option<String>,
    active_wave: Option<CourierBatchWave>,
    next_wave_sequence: u64,
}

impl CourierCollectionOrchestrator {
    pub fn new(
        collection_id: String,
        destination_parent: PathBuf,
    ) -> Result<Self, CourierCollectionError> {
        if collection_id.trim().is_empty() || !destination_parent.is_absolute() {
            return Err(collection_error(
                "COURIER_COLLECTION_EMPTY",
                "create",
                "A collection identity and absolute destination are required.",
            ));
        }
        Ok(Self {
            collection_id,
            revision: 0,
            work_phase: "collecting".to_string(),
            handoff: CourierHandoffState::default(),
            destination_parent,
            queue: CourierWorkQueue::default(),
            items: Vec::new(),
            failed_item_count: 0,
            omitted_asset_count: 0,
            last_wave_id: None,
            active_wave: None,
            next_wave_sequence: 0,
        })
    }

    pub fn add_selection(
        &mut self,
        selection: ProjectSelection,
        source_display_name: String,
    ) -> Result<String, CourierCollectionError> {
        if self.handoff.is_completed() {
            return Err(collection_error(
                "COURIER_HANDOFF_COMPLETED",
                "intake",
                "The active order has already been delivered.",
            ));
        }
        if self.handoff.is_in_progress() {
            return Err(collection_error(
                "COURIER_HANDOFF_BUSY",
                "intake",
                "The active order is being delivered.",
            ));
        }
        let item = self
            .queue
            .add(selection, source_display_name)
            .map_err(queue_error)?;
        if self.work_phase == "ready" {
            self.work_phase = "collecting".to_string();
            self.handoff.reset_for_collection_change();
        }
        Ok(item.work_item_id)
    }

    pub fn remove_item(&mut self, work_item_id: &str) -> Result<(), CourierCollectionError> {
        self.queue.remove(work_item_id).map_err(queue_error)
    }

    pub fn set_destination_parent(
        &mut self,
        destination_parent: PathBuf,
    ) -> Result<(), CourierCollectionError> {
        if self.work_phase == "processing"
            || !self.items.is_empty()
            || !destination_parent.is_absolute()
        {
            return Err(collection_error(
                "COURIER_COLLECTION_BUSY",
                "set_destination",
                "The collection destination cannot be changed now.",
            ));
        }
        self.destination_parent = destination_parent;
        Ok(())
    }

    pub fn start_processing(&mut self) -> Result<CourierBatchWave, CourierCollectionError> {
        if self.work_phase == "processing" {
            return Err(collection_error(
                "COURIER_COLLECTION_BUSY",
                "start",
                "The collection is already processing.",
            ));
        }
        if self.queue.queued_items().is_empty() {
            return Err(collection_error(
                "COURIER_COLLECTION_EMPTY",
                "start",
                "At least one queued ALS is required.",
            ));
        }
        self.work_phase = "processing".to_string();
        self.freeze_next_wave()?.ok_or_else(|| {
            collection_error(
                "COURIER_COLLECTION_EMPTY",
                "start",
                "At least one queued ALS is required.",
            )
        })
    }

    pub fn next_wave(&mut self) -> Result<Option<CourierBatchWave>, CourierCollectionError> {
        if self.work_phase != "processing" {
            return Err(collection_error(
                "COURIER_COLLECTION_BUSY",
                "next_wave",
                "The collection is not processing.",
            ));
        }
        self.freeze_next_wave()
    }

    pub fn merge_wave_result(
        &mut self,
        wave_id: &str,
        result: &BatchCopyResult,
    ) -> Result<CourierCollectionSnapshot, CourierCollectionError> {
        let wave = self
            .active_wave
            .clone()
            .ok_or_else(|| stale_wave("merge"))?;
        validate_result(&wave, wave_id, result)?;
        let updates = collect_updates(&self.queue.snapshot(), &wave, result)?;
        for update in updates {
            self.queue
                .finish_item(&update.work_item_id, &update.queue_outcome)
                .map_err(queue_error)?;
            if let Some(item) = update.collection_item {
                self.omitted_asset_count = self
                    .omitted_asset_count
                    .saturating_add(item.omitted_asset_count);
                self.items.push(item);
            } else {
                self.failed_item_count = self.failed_item_count.saturating_add(1);
            }
        }
        self.active_wave = None;
        self.revision = self.revision.saturating_add(1);
        self.handoff.reset_for_collection_change();
        self.last_wave_id = Some(wave.wave_id);
        self.work_phase = if self.queue.queued_items().is_empty() {
            "ready".to_string()
        } else {
            "processing".to_string()
        };
        Ok(self.snapshot())
    }

    pub fn snapshot(&self) -> CourierCollectionSnapshot {
        CourierCollectionSnapshot {
            schema_version: COURIER_COLLECTION_SCHEMA_VERSION.to_string(),
            collection_id: self.collection_id.clone(),
            revision: self.revision,
            work_phase: self.work_phase.clone(),
            handoff: self.handoff.snapshot(),
            destination_parent: self.destination_parent.clone(),
            queue: self.queue.snapshot(),
            items: self.items.clone(),
            completed_item_count: self
                .items
                .iter()
                .filter(|item| item.outcome == "completed")
                .count(),
            incomplete_item_count: self
                .items
                .iter()
                .filter(|item| item.outcome == "completed_incomplete")
                .count(),
            failed_item_count: self.failed_item_count,
            omitted_asset_count: self.omitted_asset_count,
            last_wave_id: self.last_wave_id.clone(),
        }
    }

    pub fn delivery_snapshot(&self) -> Result<DeliveryCollectionSnapshot, CourierCollectionError> {
        if self.work_phase != "ready" || self.items.is_empty() || !self.handoff.is_available() {
            return Err(collection_error(
                "COURIER_DELIVERY_SNAPSHOT_EMPTY",
                "delivery_snapshot",
                "No ready collection payload is available.",
            ));
        }
        Ok(DeliveryCollectionSnapshot {
            schema_version: DELIVERY_COLLECTION_SCHEMA_VERSION.to_string(),
            collection_id: self.collection_id.clone(),
            revision: self.revision,
            items: self.items.clone(),
        })
    }

    fn freeze_next_wave(&mut self) -> Result<Option<CourierBatchWave>, CourierCollectionError> {
        if self.active_wave.is_some() {
            return Err(collection_error(
                "COURIER_COLLECTION_BUSY",
                "freeze_wave",
                "An immutable courier wave is already active.",
            ));
        }
        let pending = self.queue.queued_items();
        if pending.is_empty() {
            self.work_phase = "ready".to_string();
            return Ok(None);
        }
        self.queue
            .mark_processing(
                &pending
                    .iter()
                    .map(|item| item.work_item_id.clone())
                    .collect::<Vec<_>>(),
            )
            .map_err(queue_error)?;
        self.next_wave_sequence = self.next_wave_sequence.saturating_add(1);
        let wave = CourierBatchWave {
            wave_id: format!("{}-wave-{:06}", self.collection_id, self.next_wave_sequence),
            collection_id: self.collection_id.clone(),
            base_revision: self.revision,
            selections: pending.into_iter().map(|item| item.selection).collect(),
            destination_parent: self.destination_parent.clone(),
        };
        self.active_wave = Some(wave.clone());
        Ok(Some(wave))
    }
}

fn queue_error(error: CourierQueueError) -> CourierCollectionError {
    CourierCollectionError {
        error_code: error.error_code,
        stage: error.stage,
        message: error.message,
    }
}

fn stale_wave(stage: &str) -> CourierCollectionError {
    collection_error(
        "COURIER_WAVE_STALE",
        stage,
        "The courier wave is stale or no longer active.",
    )
}

fn collection_error(code: &str, stage: &str, message: &str) -> CourierCollectionError {
    CourierCollectionError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
