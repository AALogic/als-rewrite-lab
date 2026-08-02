# ADR-007: Current-Path Copy Without Content Identity

Status: accepted
Date: 2026-08-02

## Context

The desktop MVP copies audio only when it is still present at a path recorded
by the selected ALS. In this flow the application already has a current
location binding, but it has no historical content hash from the moment when
the ALS was saved. Computing a new SHA-256 therefore cannot prove that the file
is the historical sample. Repeating that computation during preview, copy,
validation and promotion adds substantial I/O without improving the decision.

Persistent content identity belongs to the future indexed-library module. That
module may store and reuse hashes in SQLite and use them for moved-file search,
deduplication and stable identity.

## Decision

The `current_paths_copy` flow does not compute SHA-256 for audio files.

An existing safe recorded path becomes a `CurrentPathBinding`: a statement
that a regular non-symlink file of an observed size is currently available at
that exact path. It is not a `ContentIdentity` and does not claim that the file
is historically identical to the sample originally used by Ableton.

Every copy operation declares one verification policy:

```text
sha256_and_size
  used for the small source ALS and other explicitly hashed artifacts

stable_source_and_size
  used for current-path audio in the desktop MVP
```

For `stable_source_and_size`, staging must:

1. reject symlinks and non-regular sources;
2. compare the observed source size with the plan;
3. open the source and verify that its metadata is stable before and after the
   copy;
4. copy into a fresh temporary file while counting bytes;
5. sync the temporary file and verify its size;
6. atomically rename it to the planned staging path.

Validation and promotion check the exact package tree, regular non-symlink
file type and expected size. They do not read audio bytes again. The ALS,
portable manifest and private ledger retain their existing hash checks.

Contracts represent an absent audio hash as `None`, never as an empty string.
Manifest records state `content_identity_status = not_computed` and record the
verification method used.

## Consequences

- The normal desktop flow performs zero audio hash passes.
- Current-path planning no longer invokes the content-addressed AssetInventory
  or strict AssetResolution pipeline.
- Laboratory matching and future indexed search may continue to use content
  hashes through their separate contracts.
- A same-size audio replacement at the recorded path is not detected by this
  MVP policy. The package remains a copy of the file currently bound at that
  path and still requires the documented manual Ableton check.
- Original ALS and media remain read-only in every case.
