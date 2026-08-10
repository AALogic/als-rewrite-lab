# Module Spec 032: CollectionDelivery

Status: accepted for implementation, v0.2
Date: 2026-08-06
Parent capability: PC-032 / safe local collection delivery
Upstream contract: module 030 `DeliveryCollectionSnapshot v0.1`
Downstream adapter: Tauri local preferences and delivery history
Durable decision: `docs/architecture/adr/ADR-015-courier-live-collection-and-delivery.md`

## 1. Responsibility

CollectionDelivery plans and executes a source-read-only copy of every not-yet-
delivered collection item into one explicitly configured local destination.
The first product adapter targets the owner's locally mounted Google Drive
Projects directory, but the module understands only a local directory.

It owns destination collision handling, owned staging directories, recursive
regular-file copy, metadata validation, per-item isolation and delivery result.
It does not upload, authenticate, observe cloud synchronization, rewrite ALS or
modify source Project copies.

## 2. Contracts

`CollectionDeliveryRequest v0.1`:

```text
request_id
snapshot
destination_root
previously_delivered_item_ids
write_consent
```

`CollectionDeliveryPlan v0.1`:

```text
schema_version
request_id
collection_id
collection_revision
destination_root
operations
skipped_item_ids
errors
```

`CollectionDeliveryOperation v0.1`:

```text
item_id
source_root
staging_root
target_root
```

`CollectionDeliveryResult v0.1`:

```text
schema_version
request_id
run_status
collection_id
collection_revision
items
skipped_item_ids
error_codes
```

`CollectionDeliveryItemResult v0.1` records item ID, status, final local target,
file count, total bytes and an optional error code.

`CourierPreferences v0.1` stores `schema_version`, `output_parent` and optional
`local_delivery_root` in an application-private JSON file written atomically.

## 3. Safety Rules

- Plan is read-only and required before execution.
- Execution requires explicit consent and revalidates the immutable snapshot.
- Source and destination roots must be absolute, existing non-symlink
  directories. Source trees containing links or non-regular special files fail
  for that item.
- Existing destination entries are never overwritten or merged.
- Collision-safe names use a deterministic numeric suffix and are recorded in
  the plan and result.
- Copy occurs in a fresh owned sibling staging directory. Validation compares
  regular-file count, relative paths and byte sizes before atomic promotion.
- Cleanup may remove only a staging root created by this exact operation.
- One failed item does not roll back or delete another completed delivery.
- Previously delivered item IDs are skipped, preventing duplicate delivery in
  one process-lifetime collection.
- No content hash is computed in v0.1.
- Success means validated local copy only, never cloud synchronization.
- Local delivery begins one exact collection handoff. Full completion closes
  the active order; partial or failed delivery leaves a retryable package.
- A successful delivery is presented as
  `Gotowe. Zlecenie wyslane do folderu Google Drive.` The statement concerns
  the validated local sync folder, not remote cloud synchronization.
- Exported errors and aggregate reports are path-free.

## 4. Errors

```text
DELIVERY_REQUEST_INVALID
DELIVERY_CONSENT_REQUIRED
DELIVERY_DESTINATION_INVALID
DELIVERY_SOURCE_INVALID
DELIVERY_SOURCE_CONTAINS_LINK
DELIVERY_STAGING_EXISTS
DELIVERY_COPY_FAILED
DELIVERY_VALIDATION_FAILED
DELIVERY_PROMOTION_FAILED
DELIVERY_SETTINGS_INVALID
DELIVERY_SETTINGS_WRITE_FAILED
```

## 5. Gate Status Summary

```text
Product gate: CLEAR
Write-safety gate: CLEAR
No-hash verification gate: CLEAR
Cloud-sync claim gate: CLEAR; explicitly prohibited
Cross-platform local-copy gate: CLEAR
Terminal lifecycle integration gate: CLEAR
```

## 6. Acceptance

Tests prove plan-first behavior, consent, no overwrite, deterministic collision
handling, owned staging, validation, source immutability, repeated-delivery
deduplication, failure isolation, exact handoff outcome mapping, atomic settings
and path-free diagnostics.
