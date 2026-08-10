use crate::{
    BatchCopyResult, CourierBatchWave, CourierCollectionError, CourierCollectionItem,
    CourierQueueSnapshot,
};

pub(crate) struct CollectionUpdate {
    pub(crate) work_item_id: String,
    pub(crate) queue_outcome: String,
    pub(crate) collection_item: Option<CourierCollectionItem>,
}

pub(crate) fn validate_result(
    wave: &CourierBatchWave,
    wave_id: &str,
    result: &BatchCopyResult,
) -> Result<(), CourierCollectionError> {
    let identities_match = wave.wave_id == wave_id
        && result.request_id == wave.wave_id
        && result.destination_parent == wave.destination_parent
        && result.jobs.len() == wave.selections.len()
        && result
            .jobs
            .iter()
            .zip(&wave.selections)
            .all(|(job, selection)| job.selection_id == selection.selection_id);
    if !identities_match {
        return Err(stale_wave("merge"));
    }
    Ok(())
}

pub(crate) fn collect_updates(
    queue: &CourierQueueSnapshot,
    wave: &CourierBatchWave,
    result: &BatchCopyResult,
) -> Result<Vec<CollectionUpdate>, CourierCollectionError> {
    result
        .jobs
        .iter()
        .zip(&wave.selections)
        .map(|(job, selection)| update_for_job(queue, job, selection))
        .collect()
}

fn update_for_job(
    queue: &CourierQueueSnapshot,
    job: &crate::BatchCopyJobResult,
    selection: &crate::ProjectSelection,
) -> Result<CollectionUpdate, CourierCollectionError> {
    let work = queue
        .items
        .iter()
        .find(|item| item.selection.selection_id == selection.selection_id)
        .ok_or_else(|| stale_wave("merge_item"))?;
    let queue_outcome = match job.job_status.as_str() {
        "completed" => "completed",
        "completed_incomplete" => "completed_incomplete",
        "skipped_blocked" => "blocked",
        "failed" | "cancelled" => "failed",
        _ => return Err(stale_wave("merge_status")),
    };
    let collection_item = if matches!(queue_outcome, "completed" | "completed_incomplete") {
        let copy = job
            .result
            .as_ref()
            .ok_or_else(|| stale_wave("merge_result"))?;
        let target = copy
            .final_target_root
            .clone()
            .ok_or_else(|| stale_wave("merge_target"))?;
        Some(CourierCollectionItem {
            work_item_id: work.work_item_id.clone(),
            source_display_name: work.source_display_name.clone(),
            copy_result_request_id: copy.request_id.clone(),
            target_project_root: target,
            outcome: queue_outcome.to_string(),
            omitted_asset_count: copy.omitted_asset_count,
        })
    } else {
        None
    };
    Ok(CollectionUpdate {
        work_item_id: work.work_item_id.clone(),
        queue_outcome: queue_outcome.to_string(),
        collection_item,
    })
}

fn stale_wave(stage: &str) -> CourierCollectionError {
    CourierCollectionError {
        error_code: "COURIER_WAVE_RESULT_MISMATCH".to_string(),
        stage: stage.to_string(),
        message: "The batch result does not belong to the active courier wave.".to_string(),
    }
}
