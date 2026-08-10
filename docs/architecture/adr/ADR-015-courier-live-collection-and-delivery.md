# ADR-015: Courier Live Collection And Delivery

Status: accepted
Date: 2026-08-05

## Context

The compact courier currently receives one ALS, runs one module-018 copy and
binds one completed Project folder to one native drag candidate. The accepted
product direction extends that surface so the user can add ALS files before,
during and after processing, keep all successful Project copies in one logical
collection and deliver that collection without exposing native paths to React.

Module 024 already proves safe fixed-batch behavior: every preview completes
before the first write and execution is sequential. Mutating an active module-
024 request would weaken that evidence and create race-dependent behavior.

## Decision

Add a live collection layer above module 024.

```text
Open With / explicit inbound file drop
-> ordered CourierWorkQueue
-> immutable BatchWave
-> unchanged module 024
-> merge successful outputs into CourierCollection
-> immutable DeliveryCollectionSnapshot
-> native copy-only drag or planned local delivery
```

One Play action starts a processing run. The orchestrator freezes all currently
queued items into one BatchWave. ALS files accepted while that wave runs remain
pending. When the wave completes, the orchestrator atomically either freezes
the pending items into the next wave or publishes the collection as ready.
The active module-024 request is never mutated.

The collection is logical. Its items remain independent direct children of the
user-selected destination parent. Version 0.1 does not introduce a physical
batch wrapper directory. A delivery snapshot binds a collection ID, revision
and ordered list of validated output directories. New completed work advances
the collection revision but cannot change an already issued snapshot.

Work phase and payload phase are separate state axes. Presentation derives from
those facts but owns neither queue policy nor filesystem effects.

```text
work_phase: collecting | processing | ready
payload_phase: unavailable | ready | armed | dragging
```

The collection also owns a provider-independent handoff axis:

```text
handoff_status: not_started | in_progress | completed | retryable_issue
handoff_channel: none | native_drag | local_google_drive
```

The first successful handoff completes the active order. A later ALS starts a
fresh collection instead of joining completed work. Intake arriving while a
handoff is in progress is retained for that next collection. An explicit
`send again` action may reopen the same immutable delivery snapshot without
rerunning module 024.

Inbound and outbound interactions have distinct native owners. Tauri/Wry is the
sole inbound Finder drop destination. The parcel is the only outbound AppKit
drag source. Provider opening does not arm or choose payload content.

Application code must not register, unregister or rearm dragged types on
`WryWebViewParent` or its nested WebView. The outbound parcel lifecycle and an
explicit order reset leave Wry's inbound destination untouched. Every inbound
Tauri/Wry event uses the same regular-file `.als` validation before collection
intake.

The ready presentation paints the courier and held parcel as one sprite strip.
The separate outbound surface is invisible pointer geometry only. At native
drag start the held-payload strip changes to an empty-hands strip while AppKit
becomes the sole owner of the parcel image under the pointer. Cancellation or
failure restores the held strip; successful handoff keeps the parcel absent.

The compact macOS surface uses an isolated full-screen overlay adapter. Quick
mode switches the process to accessory application policy and combines
all-Spaces/all-applications/full-screen-auxiliary behavior with AppKit's
screen-saver window level. Opening the main window restores regular application
policy. Active-Space reassertion must not call `set_focus` or steal keyboard
focus. The window level is used only for ordering over ordinary full-screen
applications; login, lock-screen and Mission Control behavior is outside the
product claim.

WeTransfer remains a closed trusted URL opened in the system default browser.
The generic drag adapter carries one or more directory URLs and advertises copy
only. A successful OS drop means only that the destination accepted the native
drop; it never proves remote upload completion. Provider opening alone never
starts or completes a handoff. The product copy for an accepted native drop is
`Dostarczone pod drzwi`.

Local Google Drive delivery is a filesystem copy to an explicitly configured
local directory. It is plan-first, source-read-only, collision-safe, validated
and auditable. It proves a local copy only, not cloud synchronization. Existing
destinations are never overwritten. A deterministic absent top-level name may
be selected and must be recorded in the delivery result.

User-specific destination paths live in an atomically written local settings
file outside the repository. Source code and exported diagnostics contain no
private absolute default path.

## Consequences

- Module 024 remains unchanged and keeps its existing safety evidence.
- Dynamic intake becomes a sequence of immutable batch waves, not a mutable
  batch request.
- Completed and completed-incomplete Project copies enter the collection;
  blocked, failed, cancelled and removed items do not.
- Adding an ALS while processing schedules it automatically. Adding one after
  ready returns the courier to collecting and requires Play for the new wave.
- Earlier completed outputs remain part of the same collection across waves.
- Delivery never consumes or deletes the collection.
- React receives display-safe records and opaque IDs, not authoritative native
  paths.
- macOS implements native multi-directory drag through AppKit. Unsupported
  platforms fail closed behind the same port.
- A physical wrapper folder is considered only if a manual provider experiment
  proves that the target cannot accept a multi-directory native drag.

## Explicit Non-Goals

```text
parallel Project copying
resuming a collection after application restart
automatic upload completion detection
OAuth or remote-provider APIs
whole-computer sample search
version-family inference
deleting or moving source data
```
