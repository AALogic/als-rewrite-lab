# ADR-013: External Folder Handoff Boundary

Status: accepted
Date: 2026-08-05

## Context

QuickCopyAssistant can already create a validated complete or incomplete Ableton
Project folder through module 018. The user wants a second explicit action that
opens WeTransfer in the default browser and lets the pixel worker carry that
whole folder into the browser as a normal native drag.

Moving the Tauri window or synthesizing HTML drag events would not create an
operating-system file payload and would not let the browser expose its normal
folder-drop response. Direct upload integration would introduce credentials,
remote state and service coupling that are outside the product request.

## Decision

Add module 026 `ExternalFolderHandoff` above the existing copy result and below
the QuickCopyAssistant presentation.

```text
DesktopCopyResult
-> ExternalFolderHandoff
   -> trusted BrowserOpenerPort
   -> native FolderDragSourcePort
-> QuickCopyAssistant presentation state
```

Module 018 remains unchanged. The Tauri boundary records only its latest
successful final target as an eligible handoff candidate. React supplies an
opaque result identity and exact candidate path for revalidation, never an
arbitrary provider URL or authoritative path.

Version 0.1 supports only `wetransfer_web`, mapped in Rust to the fixed HTTPS
URL, and only a macOS AppKit native drag adapter. The drag pasteboard contains
one directory URL and advertises copy only. Every drop, cancellation or failure
consumes the opaque handoff session.

The first packaged implementation attempted to begin AppKit dragging after a
React `pointerdown` crossed IPC. Manual verification showed that the original
AppKit mouse event was no longer reliable: the worker displayed a parcel but
did not move, and the backend session could remain busy. The accepted adapter
therefore installs a narrow invisible `NSView` over the worker while armed.
That view accepts the first click after browser focus, starts the drag directly
from its real `mouseDown` event, and emits a path-free started event. React owns
only the surface rectangle and presentation state.

New copy attempts, new QuickCopy ALS launches and QuickCopy window teardown
clear the candidate, one-shot session and native surface together.

The Tauri prepare and cancel commands declare `snake_case` argument names
explicitly. This keeps the Rust IPC boundary aligned with the TypeScript call
sites and prevents pre-handler rejection of `surface_rect` or `handoff_id`.

## Consequences

- WeTransfer sees the same class of native folder drag that it receives from
  Finder, subject to owner-run browser verification.
- ALS Rescue never uploads, logs in, automates a page or observes upload
  completion.
- The existing project package is not archived, moved, deleted or modified.
- The pixel worker may visually become the drag image without making React the
  owner of the folder payload.
- A later Windows adapter can implement the same port without changing module
  025 or module 018.
- The live WeTransfer hover and downloaded-tree behavior remain explicit manual
  checks before any compatibility claim.

## Evidence

- Tauri's opener plugin opens a URL with the system default application.
- AppKit dragging sessions accept pasteboard writers such as directory file
  URLs and expose copy-only source operation masks.
- The existing quick window is already transparent and always-on-top.

References:

- <https://v2.tauri.app/reference/javascript/opener/>
- <https://developer.apple.com/documentation/appkit/nsdraggingsource>
- <https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/DragandDrop/Tasks/DraggingFiles.html>
