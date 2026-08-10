use crate::{
    BatchCopyOperations, DesktopCopyPreview, DesktopCopyResult, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest,
};

pub(crate) struct DesktopCopyOperations;

impl BatchCopyOperations for DesktopCopyOperations {
    fn prepare(&mut self, request: &DesktopPrepareCopyRequest) -> DesktopCopyPreview {
        crate::prepare_copy(request)
    }

    fn execute(&mut self, request: &DesktopExecuteCopyRequest) -> DesktopCopyResult {
        crate::execute_copy(request)
    }
}
