# Module Spec 023: ProjectCatalogApplicationService

Status: implemented and verified, v0.1
Date: 2026-08-03
Implementation target: `rescue_application` plus Tauri/React adapter
Parent capability: PC-023 / UC0 Discover And Select Ableton Projects
Upstream contracts: modules 020, 021 and 022
Durable decision: `docs/architecture/adr/ADR-011-project-catalog-identity-and-boundaries.md`

## 1. Responsibility

ProjectCatalogApplicationService composes scan, catalog build and private store,
then exposes a small path-redacted Project list and resolves explicit user
selection into concrete ALS paths for downstream application services.

```text
ProjectCatalogRefreshRequest v0.1
-> refresh_project_catalog
-> ProjectCatalogRefreshResult v0.1

ProjectCatalogListRequest v0.1
-> list_project_catalog
-> ProjectCatalogListResult v0.1

ProjectSelectionRequest v0.1
-> resolve_project_selection
-> ProjectSelectionResult v0.1
```

## 2. Flow Position

```text
platform root adapter + explicit user consent
-> ProjectCatalogApplicationService
   -> ALSProjectScanner
   -> ProjectCatalogBuilder
   -> ProjectCatalogStore
-> Project list in Desktop
-> explicit ProjectSelection
-> existing one-project analyze/copy or module 024 batch
```

The service orchestrates existing contracts. It does not reproduce scanner,
grouping, persistence, ALS analysis or package-copy policy.

## 3. Refresh Contract

`ProjectCatalogRefreshRequest`:

```text
request_id
roots
excluded_roots
store_path
max_entries
max_depth
include_backups
include_stale
```

The application receives explicit approved roots. Tauri owns platform default
root discovery and may invoke refresh only after explicit full-scan consent.

`ProjectCatalogRefreshResult`:

```text
service_version
request_id
refresh_status
scan_status
store_status
catalog
warnings
errors
```

Refresh statuses:

```text
complete
partial
cancelled
failed
```

The scanner result is always routed through modules 021 and 022. A failed scan
or store blocks projection. Partial/cancelled observations may update the store
under module 022 non-destructive rules.

## 4. List Contract

`ProjectCatalogListRequest`:

```text
store_path
include_backups
include_stale
```

`ProjectCatalogListResult`:

```text
metadata
groups
items
warnings
errors
```

`ProjectCatalogListMetadata`:

```text
service_version
catalog_revision
coverage_status
total_group_count
total_item_count
visible_item_count
hidden_backup_count
hidden_stale_count
```

`ProjectListGroup`:

```text
group_id
display_name
group_kind
item_ids
main_set_count
backup_set_count
freshness_status
```

Group kinds:

```text
project_folder
ungrouped_sets
ambiguous_project_context
```

`ProjectListItem`:

```text
item_id
live_set_id
group_id
display_name
project_display_name
location_kind
freshness_status
selection_status
modified_time_unix_ms
file_size
duplicate_display_name_count
warning_codes
```

Selection statuses:

```text
selectable
selectable_with_warning
stale_unavailable
```

The list contains no native paths, source roots or observation fingerprints.
Every item still maps one-to-one to exactly one stored LiveSet occurrence.
In short: one visible item represents one concrete LiveSet.

## 5. Projection Policy

```text
one ProjectFolder -> one group
each main LiveSet -> one selectable item
multiple main Sets remain separate; no default or primary is selected
exact Backup items are hidden unless include_backups = true
not_observed_in_latest_complete_scan items are hidden unless include_stale = true
retained_from_prior_scan remains visible with selectable_with_warning
stale items may be displayed for history but remain stale_unavailable
ungrouped Sets share a path-free synthetic group
ambiguous Sets share a path-free synthetic group and carry a warning
marker-only folders are hidden because they contain no selectable Set
same display names never merge
```

Duplicate display-name count is computed among visible items. It is evidence to
the UI that rows need date/size disambiguation; it is never identity.

## 6. Selection Contract

`ProjectSelectionRequest`:

```text
request_id
store_path
catalog_revision: required when catalog IDs are present
catalog_live_set_ids
manual_als_paths
```

`ProjectSelectionResult`:

```text
service_version
request_id
selection_status
selections
warnings
errors
```

`ProjectSelection`:

```text
selection_id
selection_source
live_set_id
native_als_path
catalog_revision
observation_fingerprint
freshness_status
```

Selection sources:

```text
catalog
manual
```

Catalog selection requires an exact current catalog revision and a non-stale
record. The service resolves the opaque ID to the stored exact path. Manual Add
ALS requires an absolute `.als` path and produces the same output contract, but
does not claim catalog identity or freshness.

Duplicate IDs, duplicate native paths, unknown IDs, stale IDs, revision drift
and invalid manual paths fail the whole selection before downstream work. The
service does not silently deduplicate or substitute a Set.

## 7. Desktop Adapter

The first Desktop screen becomes a practical Project catalog:

```text
[Scan projects] [Add ALS]
search field
optional Show backups toggle
checkbox list grouped by Project folder
selected count
[Analyze selected] for one Set
batch action supplied by module 024 for many Sets
```

On first scan the user confirms broad local scanning. Tauri chooses conservative
platform roots and exclusions, owns the private app-data store path and invokes
the application service in a blocking worker. React never receives the private
store path and never implements grouping or selection policy.

The existing manual one-ALS route remains available. The existing analysis and
copy screens remain unchanged after one selected Set is resolved.

## 8. Platform Root Adapter

Initial automatic roots:

```text
macOS
  /Users plus /Volumes when present

Windows
  available logical fixed/removable drive roots

other development hosts
  user home plus conventional mounted-media roots when present
```

Initial exclusions cover operating-system, application, recycle-bin and package
cache directories that cannot reasonably contain user Ableton projects. Every
denied or skipped area remains visible through partial coverage; the UI must not
claim that the whole machine was scanned successfully.

This adapter policy is versioned separately from module 020 traversal policy.

## 9. Errors And Warnings

`ProjectCatalogApplicationWarning`:

```text
warning_code
message
```

`ProjectCatalogApplicationError`:

```text
error_code
stage
message
```

Codes include:

```text
PROJECT_CATALOG_REFRESH_FAILED
PROJECT_CATALOG_STORE_FAILED
PROJECT_CATALOG_NOT_FOUND
PROJECT_CATALOG_LIST_INVALID
PROJECT_SELECTION_EMPTY
PROJECT_SELECTION_REVISION_REQUIRED
PROJECT_SELECTION_STALE_REVISION
PROJECT_SELECTION_UNKNOWN_ID
PROJECT_SELECTION_STALE_ITEM
PROJECT_SELECTION_DUPLICATE_ID
PROJECT_SELECTION_DUPLICATE_PATH
PROJECT_SELECTION_MANUAL_PATH_INVALID
```

No warning/error includes a native path or project filename.

## 10. Safety And Architecture

```text
no original ALS or audio writes
no ALS parsing during catalog refresh
no audio scan or hash
no UI grouping policy
no hidden primary Set
no version-family inference
no stale catalog selection
no paths in list or diagnostics
selection resolution is read-only
```

## 11. Downstream Consumer

The existing one-project Desktop flow consumes one `ProjectSelection` path.
Module 024 consumes an ordered vector of `ProjectSelection` values and remains
responsible for batch preview/execution behavior.

## 12. Acceptance Criteria

```text
refresh composes 020 -> 021 -> 022 and returns a list
backup and stale defaults are hidden and counted
retained partial records remain visible with warning
multiple main Sets remain separate selectable items
same names at different paths never merge
ambiguous and ungrouped Sets remain explicit
list and diagnostics contain no private paths
catalog IDs resolve only at matching revision
stale records cannot be selected
manual and catalog input produce one ProjectSelection contract
duplicates fail instead of being silently removed
fake batch consumer receives ordered exact selections
Tauri wire contracts and React TypeScript contracts match Rust
existing manual analysis/copy flow remains operational
tests, clippy, frontend checks and workflow_guard pass
```

## 13. Known Limits

```text
no semantic search or tags
no folder-tree view in v0.1
no version-family or primary Set recommendation
no filesystem watcher
no concurrent refresh
batch execution is delegated to implemented module 024
```

## 14. Gate Status Summary

```text
Vision Gate: CLEAR
Use Case Gate: CLEAR
Flow Gate: CLEAR
Data Gate: CLEAR
Behavior Gate: CLEAR
Decision Gate: CLEAR because one row equals one concrete Set
Safety Gate: CLEAR
Test Gate: CLEAR with synthetic stores and wire tests
Evidence Gate: CLEAR for default list projection; root policy remains adapter evidence
Module Gate: CLEAR
```
