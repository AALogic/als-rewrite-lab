# Module Spec 021: ProjectCatalogBuilder

Status: implemented and verified, v0.1
Date: 2026-08-03
Implementation target: `rescue_catalog`
Parent capability: PC-021 / UC0 Discover And Select Ableton Projects
Upstream contract: `ProjectScanResult v0.1` from module 020
Durable decision: `docs/architecture/adr/ADR-011-project-catalog-identity-and-boundaries.md`

## 1. Responsibility

ProjectCatalogBuilder is a pure transformation from raw Project scan
observations to a physical catalog snapshot.

```text
ProjectCatalogBuildRequest v0.1
-> build_project_catalog
-> ProjectCatalogSnapshot v0.1
```

It creates:

```text
ProjectFolderRecord
  one exact marker-backed physical folder occurrence

LiveSetRecord
  one concrete ALS path occurrence

BackupSet
  represented as a LiveSetRecord whose location_kind is backup

ungrouped or ambiguous LiveSetRecord
  retained without a fabricated ProjectFolder assignment
```

## 2. Flow Position

```text
ALSProjectScanner
-> ProjectCatalogBuilder
-> ProjectCatalogStore
-> ProjectCatalogApplicationService
```

The builder has no filesystem access. It consumes only the versioned public
module 020 result.

## 3. Input Contract

`ProjectCatalogBuildRequest`:

```text
snapshot_id: non-empty caller-provided snapshot identifier
scan_result: ProjectScanResult v0.1
```

Accepted upstream values:

```text
scanner_version = 0.1.0
traversal_policy_version = project_scan_v0.1
scan_status = complete | partial | cancelled
```

A failed scan cannot produce a trusted catalog snapshot. Unsupported versions,
unknown statuses, inconsistent metadata counts, duplicate observation IDs or
duplicate native observation paths fail the build.

## 4. Output Contract

`ProjectCatalogSnapshot`:

```text
metadata: ProjectCatalogMetadata
project_folders: Vec<ProjectFolderRecord>
live_sets: Vec<LiveSetRecord>
warnings: Vec<ProjectCatalogWarning>
errors: Vec<ProjectCatalogError>
```

`ProjectCatalogMetadata`:

```text
catalog_version
identity_policy_version
snapshot_id
source_scan_run_id
source_scan_status
build_status
project_folder_count
live_set_count
main_set_count
backup_set_count
ungrouped_set_count
ambiguous_set_count
source_warning_count
source_error_count
warning_count
error_count
```

Build statuses:

```text
complete
  source scan was complete and every Set association is either uniquely
  supported or explicitly ungrouped

complete_with_ambiguity
  source scan was complete but at least one Set has several structural Project
  candidates

partial
  source scan was partial or cancelled; retained records are usable only with
  explicit partial-coverage evidence

failed
  upstream or catalog contract validation failed
```

## 5. ProjectFolderRecord

```text
project_folder_id
source_root
native_root_path
marker_path
display_name
structural_status
main_set_ids
backup_set_ids
latest_modified_time_unix_ms
```

`structural_status` is `exact_marker_observed` in v0.1.

Every exact marker observation produces one physical folder record, including
a marker-only folder with no Set. Marker-only folders are retained and warned;
the later Desktop projection may hide them by default.

## 6. LiveSetRecord

```text
live_set_id
source_root
native_als_path
relative_path
display_name
file_size
modified_time_unix_ms
location_kind
association_status
project_folder_id
candidate_project_folder_ids
observation_fingerprint
```

Association:

```text
zero marker roots lexically contain the ALS
  association_status = ungrouped
  project_folder_id = None

exactly one marker root lexically contains the ALS
  association_status = structurally_associated
  project_folder_id = that ProjectFolder ID

more than one marker root lexically contains the ALS
  association_status = ambiguous_project_context
  project_folder_id = None
  candidate_project_folder_ids = every candidate in deterministic order
```

The builder does not select the nearest nested marker because module 004 treats
multiple marker-bearing ancestors as ambiguous. A later explicit product rule
may change this only through a versioned contract and evidence.

Location kinds:

```text
project_root
project_subdirectory
backup
ungrouped
backup_ungrouped
ambiguous_project_context
backup_ambiguous_project_context
```

A `Backup` path component is recognized only by exact native component equality
with `Backup` below the associated ProjectFolder root. For an ungrouped Set the
comparison starts below its scan root. A ProjectFolder named `Backup`, filename
patterns and `Backups` are not promoted into automatic rules in v0.1.

## 7. Identity Policy

Catalog IDs are local occurrence identifiers, not musical-work or audio-content
identity.

```text
identity_policy_version = catalog_native_path_sha256_v0.1

project_folder_id
  SHA-256 over a domain prefix plus the UTF-8 native marker parent path

live_set_id
  SHA-256 over a domain prefix plus the UTF-8 native ALS path

observation_fingerprint
  SHA-256 over a domain prefix, native ALS path, file size and modified time
```

The source scanner excludes non-Unicode paths in v0.1. The builder rejects a
path that cannot be represented exactly; it never uses lossy conversion.

A moved path receives a different occurrence ID. Future reconciliation across
moves is not part of this module. These path-derived IDs remain local and must
not be placed in a shareable diagnostic without a privacy review.

## 8. Grouping Rules

```text
every ALS observation produces exactly one LiveSetRecord
every marker observation produces exactly one ProjectFolderRecord
same display name at different native paths never merges
multiple main Sets in one ProjectFolder remain separate
no primary Set is selected
no version relationship is inferred
ambiguous candidate roots remain explicit
main_set_ids and backup_set_ids include only uniquely associated Sets
folder latest_modified_time uses only uniquely associated Sets
```

Project folders and Live Sets are sorted by their IDs after deterministic
path-derived ID creation. Candidate IDs and Set ID lists are also sorted.

## 9. Warnings And Errors

Warnings:

```text
CATALOG_SOURCE_SCAN_PARTIAL
CATALOG_SOURCE_SCAN_CANCELLED
CATALOG_UNGROUPED_SET_RETAINED
CATALOG_AMBIGUOUS_PROJECT_CONTEXT
CATALOG_MARKER_WITHOUT_SET
```

Warnings contain local catalog IDs, never native paths:

`ProjectCatalogWarning`:

```text
warning_id
warning_code
message
related_set_id
related_project_folder_ids
```

Errors:

```text
CATALOG_EMPTY_SNAPSHOT_ID
CATALOG_UNSUPPORTED_SCANNER_VERSION
CATALOG_UNSUPPORTED_TRAVERSAL_POLICY
CATALOG_SOURCE_SCAN_FAILED
CATALOG_UNKNOWN_SOURCE_STATUS
CATALOG_SOURCE_COUNTS_INCONSISTENT
CATALOG_DUPLICATE_OBSERVATION_ID
CATALOG_DUPLICATE_NATIVE_PATH
CATALOG_PATH_NOT_UTF8
CATALOG_PATH_NOT_ABSOLUTE
CATALOG_MARKER_ROOT_INVALID
```

`ProjectCatalogError`:

```text
error_code
message
```

## 10. Safety And Architecture

```text
pure transformation
no std::fs or platform metadata calls
no ALS parsing or decompression
no audio scanning or content hashing
no SQLite
no primary Set selection
no ProjectWork or version-family inference
no dependency, matching, package, copy or rewrite policy
no timestamps or random IDs generated during build
```

SHA-256 is used only over short local catalog identity inputs. It does not read
files and does not claim audio content identity.

## 11. Downstream Consumers

ProjectCatalogStore may consume the complete snapshot contract.

ProjectCatalogApplicationService may later consume:

```text
opaque ProjectFolder and LiveSet IDs
display names
location and association status
main/backup counts
scan coverage and warning summaries
observation fingerprints
```

It must not consume private grouping helpers or recreate association policy.

## 12. Acceptance Criteria

```text
one marker and one root Set create one associated folder and Set
multiple main Sets remain separate and no primary Set exists
Backup Set is retained and listed separately from main_set_ids
standalone ALS remains ungrouped
same display names at different paths never merge
nested marker candidates leave the Set ambiguous and unassigned
marker-only ProjectFolder is retained with a warning
partial and cancelled source coverage propagate
failed or inconsistent scanner input fails closed
path-derived IDs and observation fingerprints are deterministic
no filesystem symbols or side effects occur
fake ProjectCatalogStore consumes only the public snapshot
tests, clippy and workflow_guard pass
```

## 13. Known Limits

```text
Backup classification uses exact Backup path component only
no recommended or primary Set policy
no ProjectWork identity or version-family relationship
no ALS metadata such as tempo, tracks or Ableton version
no persistence or freshness merge
no folder tree or list projection
```

## 14. Gate Status Summary

```text
Vision Gate: CLEAR
Use Case Gate: CLEAR
Flow Gate: CLEAR
Data Gate: CLEAR
Behavior Gate: CLEAR
Decision Gate: CLEAR because ambiguity remains explicit
Safety Gate: CLEAR
Test Gate: CLEAR with pure contract fixtures
Evidence Gate: CLEAR for physical grouping policy
Module Gate: CLEAR
```
