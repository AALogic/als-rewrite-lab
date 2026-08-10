# Module Spec 029: CourierWorkQueue

Status: accepted for implementation, v0.1
Date: 2026-08-05
Parent capability: PC-029 / ordered compact-assistant intake
Upstream contract: module 023 `ProjectSelection`
Downstream consumer: module 030 `CourierCollectionOrchestrator`
Durable decision: `docs/architecture/adr/ADR-015-courier-live-collection-and-delivery.md`

## 1. Responsibility

CourierWorkQueue owns one process-lifetime ordered queue of explicit ALS work.
It accepts already resolved `ProjectSelection` values, assigns stable work-item
identities, preserves acceptance order, rejects active duplicates and permits
removal only before processing starts for that item.

It does not read ALS, prepare copies, execute module 024, decide target names,
access provider APIs or own React presentation.

## 2. Contract

`CourierWorkItem v0.1`:

```text
work_item_id
selection
source_display_name
accepted_sequence
status
```

Statuses:

```text
queued
processing
completed
completed_incomplete
blocked
failed
removed
```

`CourierQueueSnapshot v0.1`:

```text
schema_version
items
pending_item_count
```

`CourierQueueError v0.1` contains only `error_code`, `stage` and a path-free
message. Native paths stay in backend-owned `ProjectSelection` values and must
not enter exported diagnostics.

## 3. Rules

- Intake accepts only an absolute existing regular `.als` file represented by
  one `ProjectSelection`.
- Symlink ALS inputs are rejected.
- Path equality follows host semantics; Windows comparison is case-insensitive.
- The same active or completed ALS cannot be enqueued twice in one collection.
- A removed item may be added again as a new item with a new sequence.
- Removal never touches the file on disk.
- Queue order is deterministic and equals acceptance order.
- State transitions are explicit; arbitrary status strings are rejected.

## 4. Errors

```text
COURIER_SELECTION_INVALID
COURIER_SOURCE_NOT_REGULAR_ALS
COURIER_SOURCE_IS_SYMLINK
COURIER_DUPLICATE_SOURCE
COURIER_ITEM_UNKNOWN
COURIER_ITEM_NOT_REMOVABLE
COURIER_QUEUE_STATE_INVALID
```

## 5. Gate Status Summary

```text
Product gate: CLEAR
Contract gate: CLEAR
Safety gate: CLEAR
Platform gate: CLEAR for path-comparison adapter
Downstream gate: CLEAR for module 030
```

## 6. Acceptance

The module is accepted when pure tests prove validation, ordering,
deduplication, removal/re-add behavior, explicit transitions, source
immutability and path-free errors. No filesystem mutation is permitted.
