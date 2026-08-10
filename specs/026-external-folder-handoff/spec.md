# Module Spec 026: ExternalFolderHandoff

Status: v0.3 accepted as provider opener only
Date: 2026-08-05
Parent capability: PC-026 / user-initiated browser handoff of a completed Rescue Project
Upstream contracts: module 028 `TransferPayload`, module 025 `QuickCopyAssistant`
Durable decision: `docs/architecture/adr/ADR-013-external-folder-handoff-boundary.md`

## 1. Responsibility

ExternalFolderHandoff coordinates one explicit provider action and one native
drag attempt for the private payload owned by module 028.

```text
successful DesktopCopyResult
-> module 028 registers the private payload candidate
-> user chooses WeTransfer from QuickCopyAssistant
-> validate trusted provider and originating copy result identity
-> ask module 028 for one validated payload attempt
-> install a native drag surface over the rendered worker
-> open provider in the system default browser
-> arm one opaque payload attempt
-> begin one native copy-only folder drag from the surface's real mouseDown event
-> consume the attempt after drop, cancellation or failure
```

The module owns trusted-provider routing, coordination of one payload attempt
and the platform drag-source port. Module 028 owns candidate and attempt state.
Module 026 does not own project creation, upload, remote transfer state,
browser DOM access, login, link generation, archive creation or WeTransfer form
completion.

## 2. Product Boundary

Version 0.2 supports:

```text
macOS direct-download build
QuickCopyAssistant complete and incomplete successful results
one trusted provider: wetransfer_web
one final Project folder per handoff
system default browser
native AppKit folder drag
copy-only drag semantics
one-shot arm / drag / consume behavior
```

Version 0.2 does not support:

```text
ZIP creation
WeTransfer API or OAuth
automatic upload
browser extension or DOM automation
cookie or clipboard access
upload completion detection
share-link extraction
multiple providers
Windows drag adapter
batch-result folder handoff
```

Dropping a folder proves only that the native drag destination accepted the
operation. It does not prove that an upload or transfer completed.

## 3. Contracts

`ExternalFolderHandoffRequest v0.2`:

```text
request_id
provider_id
copy_result_request_id
```

The request is untrusted UI input and contains no native payload path.
`provider_id` must equal `wetransfer_web`. The backend owns the provider URL
and never accepts an arbitrary URL from React.

`ExternalFolderHandoffPrepared v0.2`:

```text
schema_version
handoff_id
provider_id
state
```

`state` is `armed`. The response contains no URL and no filesystem path.

`ExternalFolderHandoffFinished v0.2`:

```text
schema_version
handoff_id
outcome
error_code
```

Outcomes:

```text
dropped
cancelled
failed
```

`ExternalFolderHandoffError v0.2`:

```text
error_code
stage
message
```

Errors use stable codes and contain no private path.

## 4. Payload Binding

Module 028 records the candidate only after unchanged module-018 execution and
keeps its native path private. Preparing a handoff must prove through module
028:

```text
request copy_result_request_id equals the latest candidate
candidate path is absolute
candidate currently exists as a real directory
candidate itself is not a symbolic link
no other payload attempt is armed or dragging
```

React cannot nominate an arbitrary, source, staging, stale or unrelated path.

## 5. Trusted Provider Opening

Version 0.2 contains a closed provider mapping:

```text
wetransfer_web -> https://wetransfer.com/
```

The backend opens the URL with no explicit browser choice, so the operating
system selects the user's default browser. Provider opening happens only after
payload-attempt validation and native-surface installation. If opening fails,
the attempt is consumed as failed while the payload candidate remains eligible
for explicit retry.

The quick window remains visible and always-on-top. The module must not inspect
browser readiness, active tabs, page content or account state.

## 6. One-Shot Attempt State

Module-028 transitions are atomic under one mutex:

```text
no attempt -> armed -> dragging -> no attempt
```

Rules:

```text
only module 028 prepare may create armed
only the matching opaque handoff_id may begin dragging
begin consumes armed and establishes dragging before native work starts
dropped, cancelled and failed consume the attempt but preserve the candidate
old handoff IDs can never be reused
preparing a second concurrent attempt is rejected
```

Closing or replacing the quick session explicitly cancels an armed attempt.
Starting a new preview or copy, opening another ALS in QuickCopy, and closing or
destroying the QuickCopy window clear both the candidate and any armed or
dragging attempt. Cleanup also removes the native drag surface. A failed native
attempt therefore cannot leave the next WeTransfer action permanently busy.

## 7. Platform Port

`FolderDragSourcePort` receives only the already validated module-028 attempt.
Its macOS adapter:

```text
uses AppKit on the main thread
installs one invisible native NSView exactly over the rendered worker body
accepts the first click even after the default browser receives focus
starts dragging from that NSView's real AppKit mouseDown event
places one directory file URL on the native dragging pasteboard
advertises NSDragOperationCopy only
uses the packaged worker-with-parcel image as the drag image
reports the final native operation through ExternalFolderHandoffFinished
never opens, enumerates, copies, moves, deletes or archives folder contents
```

The browser must receive a normal operating-system file/folder drag, not an
HTML-simulated pointer movement. The intended manual acceptance signal is that
WeTransfer enters its own folder-drop hover state before mouse release.

React may provide only the worker's presentation rectangle. It cannot begin the
native drag, create the file URL or obtain the folder payload. Invalid,
non-finite or out-of-window rectangles fail before the provider opens.
Multi-word Tauri command arguments use an explicit `snake_case` IPC convention.
This prevents the command bridge from silently expecting `surfaceRect` or
`handoffId` while the frontend sends `surface_rect` or `handoff_id`.

Non-macOS builds retain the contracts but return
`HANDOFF_PLATFORM_UNSUPPORTED` before native drag begins.

## 8. QuickCopyAssistant Integration

The module-025 copy job remains in `complete` or `incomplete` while a separate
frontend TransferPayload projection adds presentation phases:

```text
share_opening
share_armed
share_dragging
```

Because provider activity does not replace copy-job state, every terminal
handoff outcome naturally reveals the unchanged previous copy result.

```text
complete/incomplete
  Open Folder + WeTransfer + Close

share_opening
  controls disabled while provider opening is requested

share_armed
  parcel animation and "Zabierz to ode mnie!"
  native surface over the character body awaits a real mouseDown
  window drag region is disabled

share_dragging
  entered only after the backend emits the matching native-started event
  native drag image follows the pointer
  controls disabled

dropped/cancelled/failed
  clear only the frontend payload attempt
  reveal unchanged complete/incomplete job state
  restore ordinary character window dragging
```

AssistantHost receives only the composed presentation model and controls. It
does not listen to Tauri events, call provider commands or interpret copy and
payload state.

The rendered worker remains inside the existing `96 x 128` logical-pixel box.
The parcel animation is a short deterministic CSS sprite loop and reduced
motion uses its first frame.

## 9. Safety Invariants

```text
no original ALS or audio mutation
no source, staging or arbitrary-folder handoff
no direct URL supplied by React
no move/delete drag operation
no ZIP or archive creation
no silent second drag
no remote-success claim
no path in emitted completion events or errors
no upload or browser automation
```

Module 018 behavior and public contracts remain unchanged.

## 10. Errors

Stable error codes include:

```text
HANDOFF_PROVIDER_UNSUPPORTED
HANDOFF_CANDIDATE_MISSING
HANDOFF_CANDIDATE_MISMATCH
HANDOFF_TARGET_NOT_DIRECTORY
HANDOFF_TARGET_IS_SYMLINK
HANDOFF_SESSION_BUSY
HANDOFF_SESSION_NOT_ARMED
HANDOFF_SESSION_STALE
HANDOFF_BROWSER_OPEN_FAILED
HANDOFF_NATIVE_EVENT_UNAVAILABLE
HANDOFF_DRAG_SURFACE_INVALID
HANDOFF_NATIVE_DRAG_FAILED
HANDOFF_PLATFORM_UNSUPPORTED
```

## 11. Automated Acceptance

- only module 028 may bind the latest successful complete/incomplete result;
- a new preview or execution invalidates the payload and prior attempt;
- arbitrary provider, path, source-like path, staging-like path and stale
  result requests fail before browser or native drag effects;
- attempt transitions are atomic and one-shot;
- a reset consumes a stuck dragging attempt and invalidates the payload;
- drop, cancellation and native failure consume the attempt but preserve the
  payload candidate for explicit retry;
- the native surface maps the worker rectangle into AppKit coordinates and
  rejects invalid geometry;
- only the matching native-started event may enter `share_dragging`;
- prepare and cancel commands enforce the same explicit IPC argument case;
- the macOS adapter advertises copy only and places one directory URL in the
  drag item;
- React cannot supply provider URL or call filesystem/package modules;
- QuickCopyAssistant restores the exact complete/incomplete state;
- normal character movement is unavailable only while armed or dragging;
- module 018 tests and contracts remain unchanged.

## 12. Manual Verification Pending

The first owner-run check exposed that a React pointer event reached AppKit too
late to provide a reliable native mouse event and could leave the session busy.
The implementation now uses a native NSView input surface and explicit cleanup.
These two checks must therefore be repeated on the corrected package:

```text
1. Drag from the packaged assistant over the live WeTransfer page and confirm
   that WeTransfer shows its own "Drop it like it's hot" hover treatment.
2. Complete a transfer, download it and verify that the Project directory tree
   and files are preserved without creating a ZIP in ALS Rescue.
```

Until those checks are reported, the implementation may be called ready for
manual verification but not externally verified with WeTransfer.

## 13. Gate Status

Product boundary: CONFIRMED

Architecture boundary: CONFIRMED

macOS native drag API availability: CONFIRMED

Automated implementation scope: CLEAR

Live WeTransfer behavior: MANUAL VERIFICATION PENDING, explicitly non-blocking
for implementation and blocking only for a production compatibility claim.

## 14. Provider Opener v0.3 Amendment

ExternalFolderHandoff no longer arms or owns a native payload drag. It maps one
closed provider identifier to a trusted URL and opens it in the system default
browser. Module 031 independently prepares the generic payload drag.

`ExternalProviderOpenRequest v0.1` contains `request_id` and `provider_id`.
`ExternalProviderOpenResult v0.1` contains `schema_version`, `request_id`,
`provider_id` and `state`. The only accepted provider remains
`wetransfer_web -> https://wetransfer.com/`.

Opening a page neither consumes nor arms the collection payload. It makes no
upload claim and receives no collection identity or native path.

The provider opener makes no upload claim.
