use serde::Deserialize;

#[cfg(not(target_os = "macos"))]
use crate::universal_payload_drag::{error, UniversalPayloadDragError};
#[cfg(not(target_os = "macos"))]
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct FolderDragSurfaceRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(any(target_os = "macos", test))]
pub(crate) fn validate_surface_rect(
    rect: FolderDragSurfaceRect,
    window_width: f64,
    window_height: f64,
) -> bool {
    let values = [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        window_width,
        window_height,
    ];
    values.iter().all(|value| value.is_finite())
        && rect.x >= 0.0
        && rect.y >= 0.0
        && rect.width > 0.0
        && rect.height > 0.0
        && rect.x + rect.width <= window_width + 0.5
        && rect.y + rect.height <= window_height + 0.5
}

#[cfg(not(target_os = "macos"))]
pub(crate) async fn arm_folder_drag_surface(
    _app: &AppHandle,
    _attempt_id: String,
    rect: FolderDragSurfaceRect,
) -> Result<(), UniversalPayloadDragError> {
    let _ = (rect.x, rect.y, rect.width, rect.height);
    Err(error(
        "PAYLOAD_DRAG_PLATFORM_UNSUPPORTED",
        "arm_drag",
        "Native payload dragging is not supported on this platform.",
    ))
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn remove_folder_drag_surface(_app: &AppHandle, _attempt_id: &str) {}

#[cfg(not(target_os = "macos"))]
pub(crate) fn remove_all_folder_drag_surfaces(_app: &AppHandle) {}

#[cfg(target_os = "macos")]
#[path = "macos_folder_drag_appkit.rs"]
mod appkit;

#[cfg(target_os = "macos")]
pub(crate) use appkit::{
    arm_folder_drag_surface, remove_all_folder_drag_surfaces, remove_folder_drag_surface,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parcel_surface_rect_is_validated() {
        let valid = FolderDragSurfaceRect {
            x: 12.0,
            y: 8.0,
            width: 48.0,
            height: 40.0,
        };
        let invalid = FolderDragSurfaceRect {
            x: 80.0,
            y: 8.0,
            width: 48.0,
            height: 40.0,
        };
        assert!(validate_surface_rect(valid, 96.0, 128.0));
        assert!(!validate_surface_rect(invalid, 96.0, 128.0));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn unsupported_platform_fails_closed() {
        let source = include_str!("macos_folder_drag.rs");
        assert!(source.contains("PAYLOAD_DRAG_PLATFORM_UNSUPPORTED"));
        assert!(source.contains("cfg(not(target_os = \"macos\"))"));
    }
}
