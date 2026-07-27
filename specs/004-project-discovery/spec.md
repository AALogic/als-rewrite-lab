# Module Spec 004: ProjectDiscovery

Status: ready for implementation, v0.1
Date: 2026-07-27
Implementation target: `rescue_analyzer`
Evidence: `docs/experiments/E-01-E-02-path-and-coverage-2026-07-27.md`

## Responsibility

ProjectDiscovery inspects the ancestry of one explicitly selected ALS and
records structural evidence for its Ableton Project root.

```text
ProjectDiscoveryRequest v0.1
-> ProjectDiscoveryResult v0.1
```

It answers:

```text
which ancestor directories contain an exact Ableton Project Info marker
whether exactly one structural Project root is confirmed
whether the selected Set is at root, nested, or inside Backup
whether discovery is unknown, ambiguous, confirmed, or unsupported
```

It does not recursively search the computer, parse ALS, select the main Set,
modify files, or infer a Project root from `parent(ALS)` alone.

## Product Traceability

Parent capability: Discover Project root candidates.

Flow:

```text
selected ALS
-> ProjectDiscovery
-> PathObservationContext
-> PathObservation
```

## Input

`ProjectDiscoveryRequest`:

```text
source_als_path: native path selected by the user or controlled test
```

The selected path must exist as a regular file and must not be a symlink.

## Output

`ProjectDiscoveryResult`:

```text
metadata
candidates
confirmed_project_root
discovery_status
set_location
warnings
errors
```

Statuses:

```text
confirmed
unknown
ambiguous
unsupported
```

Set locations:

```text
project_root
project_subdirectory
backup_candidate
standalone_or_unidentified
unknown
```

One exact marker-bearing ancestor confirms the root. More than one produces
`ambiguous` and no confirmed root. No marker produces `unknown` and no root.

`ProjectRootCandidate` records:

```text
candidate_path
marker_path
marker_status
depth_from_set
```

Only an actual directory named exactly `Ableton Project Info` is an accepted
marker. A symlink, file, inaccessible entry, or case variant is not accepted.

## Safety

```text
read metadata only
check one explicit marker path per ancestor
never recursively walk directories
never canonicalize or follow symlinks
never create, rename, delete, copy, or write
never mutate the selected ALS
```

## Errors

```text
PROJECT_DISCOVERY_SOURCE_NOT_FOUND
PROJECT_DISCOVERY_SOURCE_NOT_FILE
PROJECT_DISCOVERY_SOURCE_IS_SYMLINK
PROJECT_DISCOVERY_SOURCE_METADATA_FAILED
```

Marker-level failures are warnings and preserve a partial result.

## Acceptance

```text
root Set is confirmed from one exact marker
nested Set is confirmed from an ancestor marker
Backup Set is labeled without becoming a main Set
no marker never falls back to parent(ALS)
nested markers remain ambiguous
symlinks are not followed
operation is read-only and deterministic
tests, clippy and module guard pass
```
