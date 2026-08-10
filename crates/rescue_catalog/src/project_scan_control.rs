use crate::ProjectScanProgress;

pub trait ProjectScanObserver {
    fn is_cancelled(&self) -> bool;
    fn on_progress(&mut self, progress: &ProjectScanProgress);
}

impl ProjectScanObserver for () {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn on_progress(&mut self, _progress: &ProjectScanProgress) {}
}
