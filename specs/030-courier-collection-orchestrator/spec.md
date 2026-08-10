# Module Spec 030: CourierCollectionOrchestrator

Status: accepted for implementation, v0.2
Date: 2026-08-06
Parent capability: PC-030 / live courier collection
Upstream contracts: module 029 queue, module 024 batch preview/result
Downstream consumers: modules 027, 028, 031 and 032
Durable decision: `docs/architecture/adr/ADR-015-courier-live-collection-and-delivery.md`

## 1. Responsibility

CourierCollectionOrchestrator converts a live ordered queue into one or more
immutable `CourierBatchWave` values, delegates each wave to the unchanged
module 024 contract and merges successful results into one versioned logical
collection.

The implementation reuses the unchanged module 024 public behavior.

It owns collection phase, wave identity, queue-drain behavior, result mapping,
collection revision, immutable delivery snapshots and the provider-independent
handoff lifecycle of the current collection. It does not prepare or execute
one-project copies itself and does not own target naming, rewrite, validation,
provider-specific delivery or animation policy.

## 2. State And Contracts

`CourierBatchWave v0.1`:

```text
wave_id
collection_id
base_revision
selections
destination_parent
```

`CourierCollectionItem v0.1`:

```text
work_item_id
source_display_name
copy_result_request_id
target_project_root
outcome
omitted_asset_count
```

`CourierHandoffSnapshot v0.1`:

```text
schema_version
status
channel
```

`status` is `not_started`, `in_progress`, `completed` or `retryable_issue`.
`channel` is absent until a handoff starts and otherwise is `native_drag` or
`local_google_drive`.

`CourierCollectionSnapshot v0.2`:

```text
schema_version
collection_id
revision
work_phase
handoff
destination_parent
queue
items
completed_item_count
incomplete_item_count
failed_item_count
omitted_asset_count
last_wave_id
```

`DeliveryCollectionSnapshot v0.1`:

```text
schema_version
collection_id
revision
items
```

The public `CourierCollectionOrchestrator` keeps mutable runtime details
private. `work_phase` is one of `collecting`, `processing` or `ready`.
Preparation and handoff are separate state axes.

## 3. Wave Rules

- Play changes `collecting` to `processing` and freezes every currently queued
  item into one immutable wave.
- Items added during processing remain queued and cannot alter the active wave.
- After one result merge, the orchestrator atomically creates the next wave if
  queued items exist; otherwise it enters `ready`.
- The lock/transition boundary makes an item belong to exactly one wave even if
  it arrives while a prior wave is finishing.
- Adding an item after `ready` and before a handoff starts returns the work
  phase to `collecting`; existing collection items remain.
- A completed handoff is terminal for the active order. The orchestrator
  rejects direct intake until the application coordinator creates a fresh
  collection.
- Handoff transitions are bound to exact collection ID and revision. A stale
  delivery or drag result cannot complete a different collection.
- Only one handoff may be in progress. Cancelled native drag returns to
  `not_started`; failed or partial delivery returns `retryable_issue`.
- `reopen_handoff` makes the same immutable ready collection available for an
  explicit second delivery without rerunning module 024.
- Removed, blocked, failed and cancelled queue items never become collection
  items.
- Completed and completed-incomplete outputs become collection items.
- Each accepted wave result advances collection revision exactly once.
- A delivery snapshot is immutable and bound to one collection revision.
- Module 024 preview and result values are evidence to validate, not authority
  to trust. Wave, selection and destination identities must match.

## 4. Error Codes

```text
COURIER_COLLECTION_EMPTY
COURIER_COLLECTION_BUSY
COURIER_WAVE_STALE
COURIER_WAVE_RESULT_MISMATCH
COURIER_BATCH_CONTRACT_UNSUPPORTED
COURIER_COLLECTION_ITEM_INVALID
COURIER_DELIVERY_SNAPSHOT_EMPTY
COURIER_HANDOFF_BUSY
COURIER_HANDOFF_STALE
COURIER_HANDOFF_COMPLETED
COURIER_HANDOFF_TRANSITION_INVALID
```

Errors are path-free. Private collection snapshots are never exported as
diagnostic reports.

## 5. Gate Status Summary

```text
Product gate: CLEAR
Module-024 reuse gate: CLEAR
Drain-race policy gate: CLEAR
Delivery snapshot gate: CLEAR
Terminal handoff gate: CLEAR
Persistence gate: PARTIAL and explicitly outside v0.2
```

## 6. Acceptance

Tests must prove immutable waves, intake during execution, exact drain-boundary
ownership, result isolation, collection accumulation across waves, revision
binding, one active handoff, terminal completion, retry behavior, explicit
reopen without reprocessing, no active-batch mutation and no source filesystem
effects.
