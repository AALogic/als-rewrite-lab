use rescue_application::{
    BatchCopyObserver, BatchCopyResult, BatchExecuteCopyRequest, BatchPrepareCopyRequest,
    BatchProgressEvent, DesktopApplicationError,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

const BATCH_PROGRESS_EVENT: &str = "batch-copy-progress";

#[derive(Default)]
pub(crate) struct BatchCancellationState {
    flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl BatchCancellationState {
    fn begin(&self, request_id: &str) -> Result<Arc<AtomicBool>, DesktopApplicationError> {
        let mut flags = self.flags.lock().map_err(|_| state_error())?;
        if flags.contains_key(request_id) {
            return Err(adapter_error(
                "BATCH_REQUEST_ALREADY_RUNNING",
                "batch_state",
                "A batch operation with this request ID is already running.",
            ));
        }
        let flag = Arc::new(AtomicBool::new(false));
        flags.insert(request_id.to_string(), Arc::clone(&flag));
        Ok(flag)
    }

    fn cancel(&self, request_id: &str) -> Result<bool, DesktopApplicationError> {
        let flags = self.flags.lock().map_err(|_| state_error())?;
        Ok(flags.get(request_id).is_some_and(|flag| {
            flag.store(true, Ordering::Release);
            true
        }))
    }

    fn finish(&self, request_id: &str) -> Result<(), DesktopApplicationError> {
        self.flags
            .lock()
            .map_err(|_| state_error())?
            .remove(request_id);
        Ok(())
    }
}

struct TauriBatchObserver {
    app: tauri::AppHandle,
    cancelled: Arc<AtomicBool>,
}

impl BatchCopyObserver for TauriBatchObserver {
    fn on_progress(&mut self, event: &BatchProgressEvent) {
        let _ = self.app.emit(BATCH_PROGRESS_EVENT, event);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[tauri::command]
pub(crate) async fn prepare_batch_copy(
    request: BatchPrepareCopyRequest,
) -> Result<rescue_application::BatchCopyPreview, DesktopApplicationError> {
    tauri::async_runtime::spawn_blocking(move || rescue_application::prepare_batch_copy(&request))
        .await
        .map_err(|failure| background_error("batch_preview", &failure.to_string()))
}

#[tauri::command]
pub(crate) async fn execute_batch_copy(
    app: tauri::AppHandle,
    state: tauri::State<'_, BatchCancellationState>,
    request: BatchExecuteCopyRequest,
) -> Result<BatchCopyResult, DesktopApplicationError> {
    let request_id = request.request_id.clone();
    let cancelled = state.begin(&request_id)?;
    let mut observer = TauriBatchObserver { app, cancelled };
    let result = tauri::async_runtime::spawn_blocking(move || {
        rescue_application::execute_batch_copy_controlled(&request, &mut observer)
    })
    .await
    .map_err(|failure| background_error("batch_execution", &failure.to_string()));
    let finish_result = state.finish(&request_id);
    match (result, finish_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn cancel_batch_copy(
    state: tauri::State<'_, BatchCancellationState>,
    request_id: String,
) -> Result<bool, DesktopApplicationError> {
    state.cancel(&request_id)
}

fn background_error(stage: &str, message: &str) -> DesktopApplicationError {
    adapter_error(
        "BATCH_BACKGROUND_TASK_FAILED",
        stage,
        &format!("Batch background task failed: {message}"),
    )
}

fn state_error() -> DesktopApplicationError {
    adapter_error(
        "BATCH_STATE_UNAVAILABLE",
        "batch_state",
        "Batch cancellation state is unavailable.",
    )
}

fn adapter_error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::BatchCancellationState;
    use std::sync::atomic::Ordering;

    #[test]
    fn cancellation_state_is_explicit_and_removed_after_finish() {
        let state = BatchCancellationState::default();
        let flag = state.begin("batch-1").expect("start batch");
        assert!(!flag.load(Ordering::Acquire));
        assert_eq!(state.cancel("batch-1"), Ok(true));
        assert!(flag.load(Ordering::Acquire));
        assert_eq!(state.finish("batch-1"), Ok(()));
        assert_eq!(state.cancel("batch-1"), Ok(false));
    }
}
