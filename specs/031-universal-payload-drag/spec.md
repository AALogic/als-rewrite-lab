# Module Spec 031: UniversalPayloadDrag

Status: accepted for implementation, v0.2
Date: 2026-08-06
Parent capability: PC-031 / provider-independent collection drag
Upstream contract: module 028 `TransferPayloadAttempt v0.2`
Platform adapter: macOS AppKit first; unsupported platforms fail closed
Durable decision: `docs/architecture/adr/ADR-015-courier-live-collection-and-delivery.md`

## 1. Responsibility

UniversalPayloadDrag turns one backend-owned, revision-bound transfer attempt
into a platform-native copy-only drag containing one or more Project directory
URLs. It owns native surface installation, real pointer-event capture, drag
image, copy-only operation and terminal attempt events.

It does not open providers, choose payload paths, copy folder contents, upload,
automate a browser or decide whether a remote operation completed.

## 2. Contract

`UniversalPayloadDragRequest v0.1`:

```text
request_id
collection_id
collection_revision
```

React also supplies presentation-only `surface_rect`; it never supplies paths.

`UniversalPayloadDragPrepared v0.1`:

```text
schema_version
attempt_id
state
```

`UniversalPayloadDragFinished v0.1`:

```text
schema_version
attempt_id
outcome
error_code
```

Outcomes are `dropped`, `cancelled` or `failed`. `dropped` proves an accepted
native copy operation only.

## 3. Rules

- Every target is revalidated as an absolute existing non-symlink directory.
- One invalid target rejects the entire immutable attempt; partial payload drag
  is not allowed.
- The pasteboard contains all target directory URLs in snapshot order.
- The source operation mask is copy only.
- The visible native drag image is a parcel only, never the full courier.
- One native surface covers only the parcel hit rectangle.
- One attempt can begin once. Every terminal outcome consumes the attempt.
- Reset removes every native parcel surface even when the terminal attempt has
  already consumed its identity. A new courier order must expose the ordinary
  webview/Finder drop destination again.
- Starting a native drag begins the exact collection handoff. A `dropped`
  outcome completes it, `cancelled` restores it and `failed` marks a retryable
  issue. Every report is bound to the attempt collection ID and revision.
- A completed native drop produces the product presentation text
  `Dostarczone pod drzwi`; it does not claim remote upload completion.
- Provider opening and payload arming are separate user actions.
- All wire events and errors are path-free.

## 4. Errors

```text
PAYLOAD_DRAG_PLATFORM_UNSUPPORTED
PAYLOAD_DRAG_SURFACE_INVALID
PAYLOAD_DRAG_ATTEMPT_STALE
PAYLOAD_DRAG_TARGET_INVALID
PAYLOAD_DRAG_NATIVE_FAILED
```

## 5. Gate Status Summary

```text
Architecture gate: CLEAR
macOS adapter gate: CLEAR
multi-directory AppKit contract: CLEAR for implementation
live WeTransfer acceptance: MANUAL VERIFICATION PENDING
Windows adapter: outside v0.1 and fail-closed
```

## 6. Acceptance

Automated tests prove path privacy, complete multi-target validation, copy-only
semantics, surface geometry, one-shot lifecycle and handoff outcome mapping. A packaged macOS test must
prove that Finder accepts the drag. WeTransfer hover/drop remains an explicit
owner-run compatibility check and is not inferred from unit tests.
