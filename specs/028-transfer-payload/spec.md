# Module Spec 028: TransferPayload

Status: v0.2 accepted for collection payload
Date: 2026-08-05
Parent capability: PC-026 / native handoff of a completed Project folder
Supported use case: post-UC2 local folder handoff
Upstream contract: module 018 `DesktopCopyResult v0.3`
Downstream consumers: module 026 ExternalFolderHandoff and platform drag adapter
Durable decision: `docs/architecture/adr/ADR-014-assistant-host-and-transfer-payload.md`

## 1. Responsibility

TransferPayload privately binds the latest eligible completed Project folder
to its successful copy result and owns one native drag-attempt lifecycle at a
time.

```text
successful DesktopCopyResult
-> private payload candidate
-> validate originating copy result identity
-> validate current directory snapshot
-> prepare opaque attempt
-> mark matching attempt dragging
-> consume matching attempt on dropped, cancelled or failed
```

It does not own provider IDs, URLs, browser opening, React state, native AppKit
calls, upload state, share links or remote completion.

## 2. Private Payload Candidate

The candidate contains:

```text
copy_result_request_id
exact final_target_root
```

It is registered only for a successful complete or incomplete module-018
result with no errors and a non-null final target. React cannot register or
replace it.

A new preview, new execution, new QuickCopy launch or quick-window teardown
resets the candidate and any attempt together.

## 3. Attempt Contract

`TransferPayloadAttempt v0.1`:

```text
attempt_id
final_target_root (backend private)
```

The attempt path is passed only to the trusted native adapter and is never
serialized by module 028 or emitted in its diagnostics. The unchanged
module-018 `DesktopCopyResult` still contains the final target for the existing
Open Folder UI; React may display that result but cannot nominate or replace
the backend payload candidate.

State transitions:

```text
no attempt -> armed -> dragging -> no attempt
```

The candidate remains available after every terminal attempt outcome. Retrying
requires a fresh explicit prepare action and receives a new attempt ID.

## 4. Validation

Before preparing every attempt, TransferPayload proves:

```text
candidate exists
copy_result_request_id matches exactly
path is absolute
path exists as a real directory
path itself is not a symbolic link
no other attempt is active
```

It does not enumerate, hash, archive, copy, move, delete or upload directory
contents.

## 5. ExternalFolderHandoff v0.2 Integration

The module-026 request advances to:

`ExternalFolderHandoffRequest v0.2`:

```text
request_id
provider_id
copy_result_request_id
```

`final_target_root` is removed from untrusted UI input. Module 026 validates the
closed provider ID, asks TransferPayload for an attempt, arms the native
surface and opens the provider. If either effect fails, it finishes the attempt
as failed without clearing the candidate.

## 6. Errors

Stable payload errors:

```text
PAYLOAD_CANDIDATE_MISSING
PAYLOAD_CANDIDATE_MISMATCH
PAYLOAD_TARGET_MISSING
PAYLOAD_TARGET_NOT_DIRECTORY
PAYLOAD_TARGET_IS_SYMLINK
PAYLOAD_ATTEMPT_BUSY
PAYLOAD_ATTEMPT_NOT_ARMED
PAYLOAD_ATTEMPT_STALE
PAYLOAD_STATE_UNAVAILABLE
```

Errors contain stage and path-free diagnostic text.

## 7. Acceptance

- only the latest successful module-018 result becomes the candidate;
- React supplies no authoritative native target path;
- every attempt revalidates the candidate directory;
- only one attempt may be armed or dragging;
- attempt IDs are opaque, monotonic within the process and one-shot;
- dropped, cancelled and failed consume only the attempt;
- a terminal attempt leaves the candidate available for explicit retry;
- reset invalidates candidate and active attempt together;
- errors and events contain no private path;
- module 018 remains unchanged.

## 8. Gate Status

Product gate: CLEAR

Architecture gate: CLEAR

Data gate: CLEAR

Behavior gate: CLEAR

Safety gate: CLEAR

Provider catalog, delivered/reload UX and remote upload state: explicitly
outside module scope

## 9. Collection Payload v0.2 Amendment

TransferPayload now binds one immutable `DeliveryCollectionSnapshot v0.1`
rather than the latest single module-018 result. `TransferPayloadAttempt v0.2`
contains:

```text
attempt_id
collection_id
collection_revision
target_roots
```

The snapshot target list is non-empty and ordered. Every directory is validated
before an attempt is armed. One invalid member blocks the complete attempt.
Registering a newer collection revision invalidates an older candidate and
active attempt. A terminal attempt still preserves the same candidate for
explicit retry.

An explicit retry reset removes only a stale or armed attempt and its native
surface. It preserves the immutable collection candidate, so the next prepare
action receives a fresh attempt ID and revalidates every target directory.

The state machine remains:

```text
no attempt -> armed -> dragging -> no attempt
```

TransferPayload still owns no provider, browser, upload, copy or presentation
behavior. Module 031 consumes the attempt and React supplies only collection
identity, revision and presentation geometry.
