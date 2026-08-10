use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::Url;

#[derive(Debug, Clone)]
struct PendingOpenRequest {
    urls: Vec<Url>,
    launch_source: String,
}

#[derive(Debug, Default)]
pub(crate) struct StartupOpenUrlBuffer {
    ready: AtomicBool,
    pending: Mutex<Vec<PendingOpenRequest>>,
}

impl StartupOpenUrlBuffer {
    pub(crate) fn defer_if_not_ready(&self, urls: &[Url], launch_source: &str) -> bool {
        if self.ready.load(Ordering::Acquire) {
            return false;
        }
        let mut pending = match self.pending.lock() {
            Ok(pending) => pending,
            Err(poisoned) => poisoned.into_inner(),
        };
        if self.ready.load(Ordering::Acquire) {
            return false;
        }
        pending.push(PendingOpenRequest {
            urls: urls.to_vec(),
            launch_source: launch_source.to_string(),
        });
        true
    }

    pub(crate) fn mark_ready_and_drain(&self) -> Vec<(Vec<Url>, String)> {
        self.ready.store(true, Ordering::Release);
        let mut pending = match self.pending.lock() {
            Ok(pending) => pending,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *pending)
            .into_iter()
            .map(|request| (request.urls, request.launch_source))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_open_urls_are_deferred_until_runtime_is_ready() {
        let buffer = StartupOpenUrlBuffer::default();
        let Ok(first) = Url::parse("file:///tmp/first.als") else {
            panic!("first fixture URL must parse");
        };
        let Ok(second) = Url::parse("file:///tmp/second.als") else {
            panic!("second fixture URL must parse");
        };

        assert!(buffer.defer_if_not_ready(std::slice::from_ref(&first), "cold-first"));
        assert!(buffer.defer_if_not_ready(std::slice::from_ref(&second), "cold-second"));

        let pending = buffer.mark_ready_and_drain();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0], (vec![first], "cold-first".to_string()));
        assert_eq!(
            pending[1],
            (vec![second.clone()], "cold-second".to_string())
        );
        assert!(!buffer.defer_if_not_ready(&[second], "warm"));
        assert!(buffer.mark_ready_and_drain().is_empty());
    }
}
