# Module Spec 020: ALSProjectScanner

Status: implemented and verified, v0.1
Date: 2026-08-03
Implementation target: `rescue_catalog`
Parent capability: PC-020 / UC0 Discover And Select Ableton Projects
Durable decision: `docs/architecture/adr/ADR-011-project-catalog-identity-and-boundaries.md`

## 1. Responsibility

ALSProjectScanner performs a bounded, read-only walk of explicit approved
native roots and records raw filesystem observations needed by the future
physical Project catalog.

```text
ProjectScanRequest v0.1
-> scan_projects_controlled
-> ProjectScanResult v0.1
```

It answers only:

```text
which regular .als files were observed
which exact Ableton Project Info directories were observed
which approved root produced each observation
which locations were excluded, denied, skipped or not completed
whether coverage is complete, partial, cancelled or failed
```

## 2. Flow Position

```text
PermissionAndScanScope
-> ALSProjectScanner
-> ProjectCatalogBuilder
-> ProjectCatalogStore
-> ProjectCatalogApplicationService
```

Module 004 `ProjectDiscovery` remains downstream of one explicit Set selection
and establishes Project-root context for that one ALS. It is not called during
the broad catalog scan.

## 3. Input Contract

`ProjectScanRequest v0.1`:

```text
scan_run_id: non-empty caller-provided identifier
roots: one or more approved absolute native directory paths
excluded_roots: zero or more absolute native path prefixes
max_entries: positive hard traversal budget
max_depth: optional maximum directory depth relative to each root
traversal_policy_version: project_scan_v0.1
```

Rules:

```text
roots are inspected with symlink_metadata and are never canonicalized
root symlinks are rejected and never followed
missing, non-directory and unreadable roots are recorded as root errors
valid roots still run when another requested root is invalid
if no root is accepted, scan_status is failed
overlapping roots are reduced lexically so one path is not observed twice
excluded_roots are lexical exclusions and do not authorize canonicalization
max_depth None means no depth limit other than max_entries and cancellation
```

The caller owns permission UX and platform root discovery. The scanner never
expands a root upward to the home directory or to a volume.

## 4. Control And Progress Contract

`scan_projects` is a convenience entry point with a no-op observer.

`scan_projects_controlled` accepts a mutable `ProjectScanObserver`. The scanner:

```text
checks cancellation before each directory and before each entry
emits a start event
emits bounded progress after each visited directory
emits one final complete, partial, cancelled or failed event
stops without writes when cancellation is requested
returns observations collected before cancellation
```

`ProjectScanProgress v0.1` contains only aggregate values:

```text
scan_run_id
stage
requested_root_count
roots_completed
directories_visited
entries_visited
als_file_count
project_marker_count
warning_count
```

Progress events do not contain project names, file names or paths. They are safe
for direct Desktop display and do not create a high-volume path log.

## 5. Output Contract

`ProjectScanResult v0.1`:

```text
metadata: ProjectScanMetadata
als_files: Vec<ALSFileObservation>
project_markers: Vec<ProjectMarkerObservation>
warnings: Vec<ProjectScanWarning>
errors: Vec<ProjectScanError>
```

`ProjectScanMetadata`:

```text
scanner_version
traversal_policy_version
scan_run_id
scan_status
requested_root_count
accepted_root_count
directories_visited
entries_visited
als_file_count
project_marker_count
skipped_symlink_count
skipped_excluded_count
warning_count
error_count
```

Scan statuses:

```text
complete
  all readable, non-excluded entries under accepted roots were visited within
  the requested depth and entry budgets

partial
  observations are useful, but at least one root, directory or entry could not
  be inspected, or a hard traversal budget stopped the walk

cancelled
  caller cancellation stopped the walk; observations are explicitly partial

failed
  request validation or root validation left no accepted root
```

A complete result means complete only for the explicit approved roots and
exclusion policy. It never means the whole computer was scanned.

## 6. ALS File Observation

`ALSFileObservation`:

```text
observation_id
source_root
native_path
relative_path
filename
extension
file_size
modified_time_unix_ms
entry_kind
observation_status
```

Only regular files whose extension equals `als` using ASCII
case-insensitive comparison are recorded. File content is never opened.
Corrupt, empty and non-gzip files named `*.als` are still catalog candidates.

The scanner preserves Backup ALS observations. Backup classification belongs to
ProjectCatalogBuilder.

Non-Unicode paths cannot cross the current serde/JSON contract safely. They are
excluded with `PROJECT_SCAN_NON_UNICODE_PATH`, make coverage partial and remain
a documented cross-platform limitation rather than being converted lossily.

## 7. Project Marker Observation

`ProjectMarkerObservation`:

```text
marker_observation_id
source_root
project_root_candidate
marker_path
relative_marker_path
marker_name
entry_kind
observation_status
```

Only a real directory named exactly `Ableton Project Info` is a marker.
Case variants, files and symlinks are not accepted as markers.

The scanner records the marker and does not descend into it. It does not decide
whether the parent is a confirmed Project Folder; that interpretation belongs
to ProjectCatalogBuilder and, for one selected ALS, module 004.

## 8. Traversal And Exclusion Policy

Traversal is iterative and directory entries are sorted by native path before
inspection. Public ALS and marker outputs are sorted by native path and receive
deterministic sequence identifiers after sorting.

`excluded_roots` are skipped lexically. The adapter may use them for system,
cache, application-bundle or development-heavy locations. Intentional
exclusions increment `skipped_excluded_count` but do not make the scan partial;
they are part of the declared coverage policy.

`max_entries` is a hard safety budget. Reaching it before the queue is empty
produces `partial` and `PROJECT_SCAN_ENTRY_LIMIT_REACHED`.

When `max_depth` prevents descent into an observed directory, coverage is
`partial` and `PROJECT_SCAN_DEPTH_LIMIT_REACHED` is recorded.

Directory and entry symlinks are never followed. On Windows the implementation
must also reject directory entries detected as reparse-point traversal
candidates by the available native metadata boundary. Native Windows CI and a
physical NTFS experiment provide the final platform evidence.

## 9. Errors And Warnings

Request/root errors:

```text
PROJECT_SCAN_EMPTY_ID
PROJECT_SCAN_EMPTY_SCOPE
PROJECT_SCAN_INVALID_ENTRY_LIMIT
PROJECT_SCAN_UNSUPPORTED_POLICY
PROJECT_SCAN_ROOT_NOT_ABSOLUTE
PROJECT_SCAN_ROOT_NOT_FOUND
PROJECT_SCAN_ROOT_IS_SYMLINK
PROJECT_SCAN_ROOT_NOT_DIRECTORY
PROJECT_SCAN_ROOT_METADATA_FAILED
PROJECT_SCAN_NO_ACCEPTED_ROOTS
PROJECT_SCAN_EXCLUSION_NOT_ABSOLUTE
```

Traversal warnings:

```text
PROJECT_SCAN_DIRECTORY_READ_FAILED
PROJECT_SCAN_ENTRY_READ_FAILED
PROJECT_SCAN_ENTRY_METADATA_FAILED
PROJECT_SCAN_SYMLINK_SKIPPED
PROJECT_SCAN_REPARSE_POINT_SKIPPED
PROJECT_SCAN_ENTRY_LIMIT_REACHED
PROJECT_SCAN_DEPTH_LIMIT_REACHED
PROJECT_SCAN_NON_UNICODE_PATH
PROJECT_SCAN_SCOPE_INVARIANT_FAILED
```

Warnings and errors retain private native paths only inside the domain result.
Any Desktop diagnostic projection must redact them separately.

## 10. Safety Invariants

```text
read metadata only
never open or decompress ALS content
never scan audio content
never hash files
never canonicalize roots or entries
never follow symlinks or reparse-point traversal candidates
never create, copy, rename, delete or modify files
never write metadata into a discovered Project
never group Project folders or infer version families
never select a Set
never produce package, rewrite or matching decisions
```

## 11. Downstream Consumers

ProjectCatalogBuilder may consume:

```text
scan metadata and coverage
ALS native and relative paths
ALS basic metadata
exact marker observations
warnings and root errors
```

It must not depend on scanner queue order, warning prose, private helper types
or filesystem traversal implementation.

## 12. Acceptance Criteria

```text
standard Project marker and main ALS are observed
Backup ALS is retained as a raw ALS observation
standalone ALS without a marker is retained
invalid gzip bytes with an ALS extension are retained without content reads
uppercase ALS extension is accepted
non-ALS files are ignored
multiple main ALS files are all retained without primary/version inference
same names in different roots remain separate observations
overlapping roots do not duplicate lexical paths
explicit exclusions are honored and reported
symlinks are reported and never followed
entry and depth limits make coverage partial
inaccessible entries make coverage partial without discarding other roots
cancellation returns cancelled with collected observations
progress contains no private paths or names
ordering and identifiers are deterministic for unchanged input
source trees remain unchanged
fake ProjectCatalogBuilder consumes only the public result contract
tests, clippy and workflow_guard pass
```

## 13. Known Limits

```text
platform root discovery and user permission UX are upstream
non-Unicode Unix paths are reported but not cataloged in v0.1
Windows reparse behavior needs native CI and physical NTFS evidence
scan state is in memory; SQLite persistence is module 022
no resume checkpoint exists in module 020
no filesystem watcher or automatic refresh exists
no ALS metadata, dependency, sample, plugin or version-family analysis occurs
```

## 14. Gate Status Summary

```text
Vision Gate: CLEAR
Use Case Gate: CLEAR
Flow Gate: CLEAR
Data Gate: CLEAR for raw scan observations
Behavior Gate: CLEAR
Decision Gate: CLEAR because the scanner makes no product selection
Safety Gate: CLEAR
Test Gate: CLEAR with synthetic filesystem fixtures and native follow-up
Evidence Gate: PARTIAL for Windows reparse behavior, non-blocking for portable core
Module Gate: CLEAR
```
