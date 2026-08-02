# OriginalFileSize Zero In Current-Path Resolution

Status: confirmed product interpretation for current-path binding

Date: 2026-08-02

## Observation

A real Live 11.3 Set produced a complete read-only dependency report but its
current-path copy preview was blocked. Four requirements pointed to existing
regular files at their exact recorded paths while carrying:

```text
OriginalFileSize = 0
OriginalCrc = 0
```

The files were non-empty and usable as the project's current path bindings.
PathObservation already interpreted non-positive expected size as unavailable,
but AssetResolution interpreted zero as a literal expected byte size. That
cross-module semantic drift created `file_size_differs` conflicts.

## Decision

AssetResolution preserves the raw string but scores expected size only when it
parses to a positive integer. Zero provides no size-match evidence and creates
no mismatch conflict. This decision authorizes current-path binding only; it
does not prove historical sample identity or define the general Ableton CRC
algorithm.

## Evidence

Before correction:

```text
required assets: 30
blocked by resolution_not_auto_accepted: 4
```

After correction on the same ALS snapshot:

```text
preview status: complete_copy_preview_ready
required assets: 30
unique audio copies: 29
rewrite operations: 3744
omissions: 0
preview filesystem writes: 0
```

The duplicate copy count is expected because two requirement groups bind to
the same path and content. PackagePlanner deduplicates the copied content while
retaining every rewrite occurrence.
