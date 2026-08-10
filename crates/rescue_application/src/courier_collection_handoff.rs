use super::{collection_error, CourierCollectionError, CourierCollectionOrchestrator};
use serde::{Deserialize, Serialize};

const HANDOFF_SCHEMA_VERSION: &str = "0.1";
const NOT_STARTED: &str = "not_started";
const IN_PROGRESS: &str = "in_progress";
const COMPLETED: &str = "completed";
const RETRYABLE_ISSUE: &str = "retryable_issue";
const NATIVE_DRAG: &str = "native_drag";
const LOCAL_GOOGLE_DRIVE: &str = "local_google_drive";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierHandoffSnapshot {
    pub schema_version: String,
    pub status: String,
    pub channel: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct CourierHandoffState {
    status: String,
    channel: Option<String>,
}

impl Default for CourierHandoffState {
    fn default() -> Self {
        Self {
            status: NOT_STARTED.to_string(),
            channel: None,
        }
    }
}

impl CourierHandoffState {
    pub(super) fn snapshot(&self) -> CourierHandoffSnapshot {
        CourierHandoffSnapshot {
            schema_version: HANDOFF_SCHEMA_VERSION.to_string(),
            status: self.status.clone(),
            channel: self.channel.clone(),
        }
    }

    pub(super) fn is_completed(&self) -> bool {
        self.status == COMPLETED
    }

    pub(super) fn is_in_progress(&self) -> bool {
        self.status == IN_PROGRESS
    }

    pub(super) fn is_available(&self) -> bool {
        matches!(self.status.as_str(), NOT_STARTED | RETRYABLE_ISSUE)
    }

    pub(super) fn reset_for_collection_change(&mut self) {
        self.status = NOT_STARTED.to_string();
        self.channel = None;
    }
}

impl CourierCollectionOrchestrator {
    pub fn begin_handoff(
        &mut self,
        collection_id: &str,
        revision: u64,
        channel: &str,
    ) -> Result<(), CourierCollectionError> {
        self.validate_handoff_identity(collection_id, revision)?;
        validate_channel(channel)?;
        if self.work_phase != "ready" || self.items.is_empty() {
            return Err(collection_error(
                "COURIER_HANDOFF_TRANSITION_INVALID",
                "begin_handoff",
                "Only a ready non-empty collection can begin handoff.",
            ));
        }
        if self.handoff.is_completed() {
            return Err(completed_error("begin_handoff"));
        }
        if self.handoff.is_in_progress() {
            return Err(busy_error("begin_handoff"));
        }
        self.handoff.status = IN_PROGRESS.to_string();
        self.handoff.channel = Some(channel.to_string());
        Ok(())
    }

    pub fn complete_handoff(
        &mut self,
        collection_id: &str,
        revision: u64,
        channel: &str,
    ) -> Result<(), CourierCollectionError> {
        self.finish_handoff(
            collection_id,
            revision,
            channel,
            COMPLETED,
            "complete_handoff",
        )
    }

    pub fn mark_handoff_retryable(
        &mut self,
        collection_id: &str,
        revision: u64,
        channel: &str,
    ) -> Result<(), CourierCollectionError> {
        self.finish_handoff(
            collection_id,
            revision,
            channel,
            RETRYABLE_ISSUE,
            "retry_handoff",
        )
    }

    pub fn cancel_handoff(
        &mut self,
        collection_id: &str,
        revision: u64,
        channel: &str,
    ) -> Result<(), CourierCollectionError> {
        self.finish_handoff(
            collection_id,
            revision,
            channel,
            NOT_STARTED,
            "cancel_handoff",
        )?;
        self.handoff.channel = None;
        Ok(())
    }

    pub fn reopen_handoff(&mut self) -> Result<(), CourierCollectionError> {
        if !self.handoff.is_completed() || self.work_phase != "ready" || self.items.is_empty() {
            return Err(collection_error(
                "COURIER_HANDOFF_TRANSITION_INVALID",
                "reopen_handoff",
                "Only a completed ready collection can be sent again.",
            ));
        }
        self.handoff.reset_for_collection_change();
        Ok(())
    }

    pub fn handoff_completed(&self) -> bool {
        self.handoff.is_completed()
    }

    fn finish_handoff(
        &mut self,
        collection_id: &str,
        revision: u64,
        channel: &str,
        target_status: &str,
        stage: &str,
    ) -> Result<(), CourierCollectionError> {
        self.validate_handoff_identity(collection_id, revision)?;
        validate_channel(channel)?;
        if !self.handoff.is_in_progress() || self.handoff.channel.as_deref() != Some(channel) {
            return Err(collection_error(
                "COURIER_HANDOFF_TRANSITION_INVALID",
                stage,
                "The handoff result does not match the active handoff.",
            ));
        }
        self.handoff.status = target_status.to_string();
        Ok(())
    }

    fn validate_handoff_identity(
        &self,
        collection_id: &str,
        revision: u64,
    ) -> Result<(), CourierCollectionError> {
        if self.collection_id != collection_id || self.revision != revision {
            return Err(collection_error(
                "COURIER_HANDOFF_STALE",
                "handoff_identity",
                "The handoff does not match the active collection revision.",
            ));
        }
        Ok(())
    }
}

fn validate_channel(channel: &str) -> Result<(), CourierCollectionError> {
    if matches!(channel, NATIVE_DRAG | LOCAL_GOOGLE_DRIVE) {
        return Ok(());
    }
    Err(collection_error(
        "COURIER_HANDOFF_TRANSITION_INVALID",
        "handoff_channel",
        "The handoff channel is unsupported.",
    ))
}

fn busy_error(stage: &str) -> CourierCollectionError {
    collection_error(
        "COURIER_HANDOFF_BUSY",
        stage,
        "Another handoff is already in progress.",
    )
}

fn completed_error(stage: &str) -> CourierCollectionError {
    collection_error(
        "COURIER_HANDOFF_COMPLETED",
        stage,
        "The active order has already been delivered.",
    )
}
