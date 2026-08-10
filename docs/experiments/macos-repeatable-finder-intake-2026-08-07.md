# macOS Repeatable Finder Intake Investigation

## Question

Why does the courier accept a Finder-dropped ALS in the first order but stop
accepting Finder drops after a successful handoff followed by `Nowe zlecenie`?

## Reproduction Status

Owner runtime testing reproduces the failure. Static tests alone are not
accepted as closure. The final acceptance sequence remains:

1. Drop an ALS from Finder.
2. Process the order.
3. Complete native parcel handoff.
4. Select `Nowe zlecenie`.
5. Drop another ALS from Finder.
6. Confirm that a fresh one-item order is created.

## Evidence

### Product State

The collection reset replaces the runtime store with its default value and
resets the outbound payload attempt. A secondary Open With event can add a new
ALS after reset. This rules out the collection queue as the primary failure.

### AppKit Contract

Apple documents two requirements for a drag destination: it registers the
pasteboard types it accepts and implements the `NSDraggingDestination`
messages. `unregisterDraggedTypes` only unregisters a possible destination; it
is not documented as a stale-state reset or rearm operation.

Sources:

- https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/DragandDrop/Concepts/dragdestination.html
- https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/DragandDrop/Tasks/acceptingdrags.html
- https://developer.apple.com/documentation/appkit/nsview/unregisterdraggedtypes%28%29?language=objc

### Installed Tauri/Wry Behavior

The exact locked runtime is Tauri 2.11.5, tauri-runtime-wry 2.11.4 and Wry
0.55.1. Tauri enables its Wry drag-drop handler by default. Wry implements the
macOS destination callbacks on `WryWebView`; `WryWebViewParent` is a wrapper
view and does not own Wry's file-drop callback.

Relevant local dependency sources:

- `tauri-runtime-wry-2.11.4/src/lib.rs`
- `wry-0.55.1/src/wkwebview/class/wry_web_view.rs`
- `wry-0.55.1/src/wkwebview/class/wry_web_view_parent.rs`
- `wry-0.55.1/src/wkwebview/drag_drop.rs`

Official references:

- https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
- https://github.com/tauri-apps/wry

### Live Hierarchy

LLDB inspection of the installed application found both the parent and nested
WebView registered for `NSFilenamesPboardType`. The parent registration came
from the application adapter, while the nested WebView owns Wry's destination
implementation. The adapter therefore created a competing registered object
that could not forward paths through Wry's callback.

After removing the adapter, a fresh installed build showed empty
`registeredDraggedTypes` arrays on every `WryWebViewParent`. The nested
`WryWebView` instances still advertised `NSFilenamesPboardType` and their normal
WebKit types. This confirms that the repair removed the competing registration
without disabling Wry's real Finder destination.

## Root Cause

The application tried to repair framework-owned inbound state by repeatedly
calling `unregisterDraggedTypes` and `registerForDraggedTypes` on both Wry
views. The underlying assumption was invalid: preserving a registered type does
not preserve or create the callback that handles the drag. Source-shape tests
verified the re-registration sequence, not second-cycle event delivery.

## Decision

Tauri/Wry is the sole owner of inbound Finder file-drop registration and event
delivery. Application code:

- does not add an AppKit inbound destination;
- does not register, unregister or rearm Wry dragged types;
- receives Tauri `WindowEvent::DragDrop` and uses the shared ALS validator;
- keeps the outbound AppKit parcel source as a separate responsibility;
- resets collection and outbound payload state without mutating inbound state.

## Dependency Upgrade Assessment

Wry 0.56.0 does not document a fix for this second-cycle defect. Open Wry PR
1723 improves pasteboard decoding for sources that publish only
`public.file-url`; it does not justify an upgrade as a repair for the competing
destination created by this application.

References:

- https://github.com/tauri-apps/wry/releases/tag/wry-v0.56.0
- https://github.com/tauri-apps/wry/pull/1723

## Verification Boundary

Automated checks can prove the ownership rule, source validation and reset
contract. Only a packaged macOS Finder interaction can prove the complete
second-cycle behavior. That runtime check remains open until performed.
