use super::{validate_surface_rect, FolderDragSurfaceRect};
use crate::transfer_payload::{TransferPayloadAttempt, TransferPayloadState};
use crate::universal_payload_drag::{error, UniversalPayloadDragError};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadOnly};
use objc2_app_kit::{
    NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession, NSDraggingSource,
    NSEvent, NSImage, NSPasteboardWriting, NSView,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSData, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const DRAG_IMAGE: &[u8] = include_bytes!("../assets/quick-parcel-drag.png");
const DRAG_IMAGE_SIZE: f64 = 48.0;
const INSTALL_TIMEOUT: Duration = Duration::from_secs(3);

struct DragSurfaceIvars {
    app: AppHandle,
    attempt_id: String,
}

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    #[ivars = DragSurfaceIvars]
    struct MacFolderDragSurface;

    impl MacFolderDragSurface {
        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
            true
        }

        #[unsafe(method(mouseDownCanMoveWindow))]
        fn mouse_down_can_move_window(&self) -> bool {
            false
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            if let Err(failure) = start_drag(self, event) {
                crate::universal_payload_drag_commands::finish_native_drag(
                    &self.ivars().app,
                    &self.ivars().attempt_id,
                    "failed",
                    Some(&failure.error_code),
                );
                release_surface(&self.ivars().attempt_id);
            }
        }
    }

    unsafe impl NSObjectProtocol for MacFolderDragSurface {}

    unsafe impl NSDraggingSource for MacFolderDragSurface {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn source_operation_mask(
            &self,
            _session: &NSDraggingSession,
            _context: NSDraggingContext,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(draggingSession:endedAtPoint:operation:))]
        fn dragging_ended(
            &self,
            _session: &NSDraggingSession,
            _screen_point: NSPoint,
            operation: NSDragOperation,
        ) {
            let outcome = if operation.contains(NSDragOperation::Copy) {
                "dropped"
            } else if operation == NSDragOperation::None {
                "cancelled"
            } else {
                "failed"
            };
            let code = (outcome == "failed").then_some("PAYLOAD_DRAG_NATIVE_FAILED");
            crate::universal_payload_drag_commands::finish_native_drag(
                &self.ivars().app,
                &self.ivars().attempt_id,
                outcome,
                code,
            );
            release_surface(&self.ivars().attempt_id);
        }
    }
);

impl MacFolderDragSurface {
    fn new(
        app: AppHandle,
        attempt_id: String,
        frame: NSRect,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(DragSurfaceIvars { app, attempt_id });
        // SAFETY: NSView's designated frame initializer is valid for this subclass.
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

thread_local! {
    static ACTIVE_SURFACES: RefCell<HashMap<String, Retained<MacFolderDragSurface>>> =
        RefCell::new(HashMap::new());
}

pub(crate) async fn arm_folder_drag_surface(
    app: &AppHandle,
    attempt_id: String,
    rect: FolderDragSurfaceRect,
) -> Result<(), UniversalPayloadDragError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    let scheduled_app = app.clone();
    app.run_on_main_thread(move || {
        let result = install_on_main_thread(&scheduled_app, attempt_id, rect);
        let _ = sender.send(result);
    })
    .map_err(|_| native_error("The native drag surface could not be scheduled."))?;
    tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(INSTALL_TIMEOUT))
        .await
        .map_err(|failure| native_error(&format!("Native drag setup failed: {failure}")))?
        .map_err(|_| native_error("The native drag surface timed out."))?
}

pub(crate) fn remove_folder_drag_surface(app: &AppHandle, attempt_id: &str) {
    let id = attempt_id.to_string();
    let _ = app.run_on_main_thread(move || release_surface(&id));
}

pub(crate) fn remove_all_folder_drag_surfaces(app: &AppHandle) {
    let _ = app.run_on_main_thread(release_all_surfaces);
}

fn install_on_main_thread(
    app: &AppHandle,
    attempt_id: String,
    rect: FolderDragSurfaceRect,
) -> Result<(), UniversalPayloadDragError> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| native_error("The drag surface did not run on the AppKit main thread."))?;
    let window = app
        .get_webview_window(crate::quick_copy_entry::QUICK_WINDOW_LABEL)
        .ok_or_else(|| native_error("The quick-copy window is unavailable."))?;
    let raw_view = window
        .ns_view()
        .map_err(|_| native_error("The quick-copy native view is unavailable."))?;
    // SAFETY: Tauri owns a valid NSView while the webview window exists.
    let view = unsafe { Retained::retain(raw_view.cast::<NSView>()) }
        .ok_or_else(|| native_error("The quick-copy native view is unavailable."))?;
    let frame = native_frame(rect, view.bounds(), view.isFlipped())?;
    let surface = MacFolderDragSurface::new(app.clone(), attempt_id.clone(), frame, mtm);
    release_all_surfaces();
    view.addSubview(&surface);
    ACTIVE_SURFACES.with(|surfaces| {
        surfaces.borrow_mut().insert(attempt_id, surface);
    });
    Ok(())
}

fn start_drag(
    surface: &MacFolderDragSurface,
    event: &NSEvent,
) -> Result<(), UniversalPayloadDragError> {
    let app = &surface.ivars().app;
    let attempt_id = &surface.ivars().attempt_id;
    let drag = app
        .state::<TransferPayloadState>()
        .begin_attempt(attempt_id)?;
    if let Err(failure) = crate::courier_commands::begin_courier_handoff(
        app,
        &drag.collection_id,
        drag.collection_revision,
        crate::courier_commands::NATIVE_HANDOFF_CHANNEL,
    ) {
        let _ = app
            .state::<TransferPayloadState>()
            .finish_attempt(attempt_id);
        return Err(error(
            "PAYLOAD_DRAG_ATTEMPT_STALE",
            "begin_handoff",
            &failure.message,
        ));
    }
    crate::universal_payload_drag_commands::emit_native_drag_started(app, attempt_id);
    let items = dragging_items(&drag, event)?;
    let references = items.iter().map(|item| &**item).collect::<Vec<_>>();
    let array = NSArray::from_slice(&references);
    let source: &ProtocolObject<dyn NSDraggingSource> = ProtocolObject::from_ref(surface);
    surface.beginDraggingSessionWithItems_event_source(&array, event, source);
    Ok(())
}

fn dragging_items(
    drag: &TransferPayloadAttempt,
    event: &NSEvent,
) -> Result<Vec<Retained<NSDraggingItem>>, UniversalPayloadDragError> {
    let image = drag_image()?;
    drag.target_roots
        .iter()
        .enumerate()
        .map(|(index, target)| dragging_item(target, event, &image, index))
        .collect()
}

fn dragging_item(
    target: &std::path::Path,
    event: &NSEvent,
    image: &NSImage,
    index: usize,
) -> Result<Retained<NSDraggingItem>, UniversalPayloadDragError> {
    let path = NSString::from_str(&target.to_string_lossy());
    let file_url = NSURL::fileURLWithPath_isDirectory(&path, true);
    let writer: &ProtocolObject<dyn NSPasteboardWriting> = ProtocolObject::from_ref(&*file_url);
    let item = NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), writer);
    let point = event.locationInWindow();
    let offset = (index.min(3) as f64) * 3.0;
    let frame = NSRect::new(
        NSPoint::new(
            point.x - DRAG_IMAGE_SIZE / 2.0 + offset,
            point.y - DRAG_IMAGE_SIZE / 2.0 - offset,
        ),
        NSSize::new(DRAG_IMAGE_SIZE, DRAG_IMAGE_SIZE),
    );
    // SAFETY: NSImage is a valid dragging-frame contents object.
    unsafe { item.setDraggingFrame_contents(frame, Some(image)) };
    Ok(item)
}

fn drag_image() -> Result<Retained<NSImage>, UniversalPayloadDragError> {
    let data = NSData::with_bytes(DRAG_IMAGE);
    NSImage::initWithData(NSImage::alloc(), &data)
        .ok_or_else(|| native_error("The packaged parcel image could not be decoded."))
}

fn native_frame(
    rect: FolderDragSurfaceRect,
    bounds: NSRect,
    flipped: bool,
) -> Result<NSRect, UniversalPayloadDragError> {
    if !validate_surface_rect(rect, bounds.size.width, bounds.size.height) {
        return Err(error(
            "PAYLOAD_DRAG_SURFACE_INVALID",
            "arm_drag",
            "The parcel drag surface is outside the quick-copy window.",
        ));
    }
    let y = if flipped {
        bounds.origin.y + rect.y
    } else {
        bounds.origin.y + bounds.size.height - rect.y - rect.height
    };
    Ok(NSRect::new(
        NSPoint::new(bounds.origin.x + rect.x, y),
        NSSize::new(rect.width, rect.height),
    ))
}

fn release_surface(attempt_id: &str) {
    ACTIVE_SURFACES.with(|surfaces| {
        if let Some(surface) = surfaces.borrow_mut().remove(attempt_id) {
            surface.removeFromSuperview();
        }
    });
}

fn release_all_surfaces() {
    ACTIVE_SURFACES.with(|surfaces| {
        for (_, surface) in surfaces.borrow_mut().drain() {
            surface.removeFromSuperview();
        }
    });
}

fn native_error(message: &str) -> UniversalPayloadDragError {
    error("PAYLOAD_DRAG_NATIVE_FAILED", "arm_drag", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dom_surface_rect_is_converted_to_unflipped_appkit_coordinates() {
        let bounds = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(300.0, 250.0));
        let rect = FolderDragSurfaceRect {
            x: 120.0,
            y: 90.0,
            width: 48.0,
            height: 48.0,
        };
        assert_eq!(
            native_frame(rect, bounds, false).map(|frame| frame.origin),
            Ok(NSPoint::new(120.0, 112.0))
        );
    }
}
