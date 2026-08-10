# Module Spec 022: ProjectCatalogStore

Status: implemented and verified, v0.1
Date: 2026-08-03
Implementation target: `rescue_catalog`
Parent capability: PC-022 / UC0 Discover And Select Ableton Projects
Upstream contract: `ProjectCatalogSnapshot v0.1` from module 021
Durable decision: `docs/architecture/adr/ADR-011-project-catalog-identity-and-boundaries.md`

## 1. Responsibility

ProjectCatalogStore persists the private physical Project catalog and preserves
record freshness across complete, partial and cancelled scans.

```text
ProjectCatalogStoreRequest v0.1
-> store_project_catalog
-> ProjectCatalogStoreResult v0.1

store path
-> load_project_catalog
-> ProjectCatalogLoadResult v0.1
```

The v0.1 adapter stores one versioned JSON state file. Its public contract does
not expose JSON details, so a later SQLite adapter can replace it without
changing catalog-building or Desktop projection policy.

## 2. Flow Position

```text
ALSProjectScanner
-> ProjectCatalogBuilder
-> ProjectCatalogStore
-> ProjectCatalogApplicationService
```

This module performs private local persistence. It does not scan the filesystem,
parse ALS, group projects, select a primary Set or create a Desktop list.

## 3. Input Contract

`ProjectCatalogStoreRequest`:

```text
store_path: explicit native path to the private catalog JSON file
coverage_scope_id: path-redacted deterministic identity of roots and exclusions
snapshot: successful ProjectCatalogSnapshot v0.1
```

Accepted snapshot statuses:

```text
complete
complete_with_ambiguity
partial
```

`failed`, unknown versions, inconsistent counts, empty IDs and invalid record
references are rejected before any write.

The caller owns platform app-data-directory selection. The Store never invents
a home-directory or global path.

## 4. Persisted State

`StoredProjectCatalog`:

```text
metadata: ProjectCatalogStoreMetadata
project_folders: Vec<StoredProjectFolderRecord>
live_sets: Vec<StoredLiveSetRecord>
catalog_warnings: Vec<ProjectCatalogWarning>
```

`ProjectCatalogStoreMetadata`:

```text
storage_schema_version
revision
coverage_scope_id
last_snapshot_id
last_scan_run_id
last_scan_status
coverage_status
project_folder_count
live_set_count
observed_project_folder_count
observed_live_set_count
retained_project_folder_count
retained_live_set_count
stale_project_folder_count
stale_live_set_count
```

`StoredProjectFolderRecord` and `StoredLiveSetRecord` each contain:

```text
record: complete module 021 record
freshness_status
last_observed_scan_run_id
```

Freshness statuses:

```text
observed_in_latest_scan
retained_from_prior_scan
not_observed_in_latest_complete_scan
```

Coverage statuses:

```text
complete
partial
cancelled
```

## 5. Merge Policy

Every record is keyed only by the stable local occurrence ID produced by module
021. Display names never identify or merge records.

For a complete source scan:

```text
incoming record
  replace the prior record and mark observed_in_latest_scan

prior record absent from incoming snapshot
  when coverage_scope_id is unchanged, retain it and mark
  not_observed_in_latest_complete_scan

  when coverage_scope_id changed, retain it as retained_from_prior_scan because
  the new scan did not prove absence under the prior scope
```

For a partial or cancelled source scan:

```text
incoming record
  update it and mark observed_in_latest_scan

prior observed record absent from incoming snapshot
  retain it and mark retained_from_prior_scan

prior not_observed_in_latest_complete_scan record absent from incoming snapshot
  preserve that stronger stale evidence
```

A partial update of an existing ProjectFolder unions known main and Backup Set
IDs instead of shrinking membership from incomplete evidence. A complete update
replaces those lists exactly.

No update deletes history in v0.1. Future compaction requires a separate policy.

## 6. Output Contracts

`ProjectCatalogStoreResult`:

```text
operation_status
catalog: Option<StoredProjectCatalog>
warnings: Vec<ProjectCatalogStoreWarning>
errors: Vec<ProjectCatalogStoreError>
```

Operation statuses:

```text
stored
already_current
failed
```

`ProjectCatalogLoadResult`:

```text
load_status
catalog: Option<StoredProjectCatalog>
errors: Vec<ProjectCatalogStoreError>
```

Load statuses:

```text
loaded
not_found
failed
```

No private path is copied into warning or error messages.

## 7. Storage And Atomicity

```text
serialize deterministic pretty JSON plus final newline
validate parent and target are not symlinks
write a create-new temporary sibling
flush and sync the temporary file
atomically replace or create the final state file
sync the parent directory where supported
remove module-owned temporary/backup files on success
load and validate the just-written state
return stored only after exact round trip
```

On Unix, same-directory rename provides atomic replacement. On Windows x64,
the adapter uses the existing project pattern based on `ReplaceFileW` and a
same-directory backup. Unsupported replacement platforms fail closed.

If validation, serialization or persistence fails, a prior valid state remains
unchanged. Corrupt or unsupported existing state is never overwritten silently.

## 8. Idempotency And Determinism

Repeating the same snapshot against the same state returns `already_current`,
does not increment revision and does not rewrite the file.

Revision starts at 1 and increments only for a semantic state change. Ordering
of folder and Set records is deterministic by occurrence ID. Set-ID lists are
sorted and deduplicated.

## 9. Errors And Warnings

Errors include:

```text
CATALOG_STORE_INPUT_INVALID
CATALOG_STORE_PATH_INVALID
CATALOG_STORE_PARENT_INVALID
CATALOG_STORE_TARGET_SYMLINK
CATALOG_STORE_READ_FAILED
CATALOG_STORE_DESERIALIZE_FAILED
CATALOG_STORE_SCHEMA_UNSUPPORTED
CATALOG_STORE_STATE_INVALID
CATALOG_STORE_SERIALIZE_FAILED
CATALOG_STORE_TEMP_CREATE_FAILED
CATALOG_STORE_TEMP_WRITE_FAILED
CATALOG_STORE_ATOMIC_REPLACE_FAILED
CATALOG_STORE_ROUND_TRIP_FAILED
```

Warnings:

```text
CATALOG_STORE_PARTIAL_COVERAGE_RETAINED
CATALOG_STORE_CANCELLED_COVERAGE_RETAINED
CATALOG_STORE_SCOPE_CHANGED_RETAINED
```

## 10. Safety And Architecture

```text
private local state only
no original ALS or audio writes
no discovered-path mutation
no hidden primary Set or version-family selection
no SQLite in v0.1
no audio hash or ALS content read
no deletion or garbage collection
no silent recovery from corrupt state
no UI policy
```

## 11. Downstream Consumer

ProjectCatalogApplicationService consumes only `StoredProjectCatalog` and its
freshness evidence. It does not parse the JSON file or reproduce merge policy.

## 12. Acceptance Criteria

```text
first valid snapshot persists and round-trips
identical replay is idempotent
complete absence marks records stale but retains them
partial/cancelled absence retains prior evidence without a deletion claim
partial folder updates cannot shrink known membership
same occurrence ID with changed observation updates the record
corrupt and unsupported stores fail closed without overwrite
failed catalog snapshots do not write
atomic success leaves no temporary or backup files
stored ordering and bytes are deterministic
errors and warnings contain no private paths
fake application consumer uses only public StoredProjectCatalog
tests, clippy and workflow_guard pass
```

## 13. Known Limits

```text
one local writer is assumed in v0.1
no concurrent-process locking
no SQLite query index
no filesystem watcher
no compaction or history deletion
no moved-path reconciliation
```

## 14. Gate Status Summary

```text
Vision Gate: CLEAR
Use Case Gate: CLEAR
Flow Gate: CLEAR
Data Gate: CLEAR
Behavior Gate: CLEAR
Decision Gate: CLEAR because incomplete absence remains non-destructive
Safety Gate: CLEAR
Test Gate: CLEAR with temporary private store fixtures
Evidence Gate: CLEAR for JSON MVP adapter and atomic platform pattern
Module Gate: CLEAR
```
