# Module Spec 007: AssetInventory

Status: ready for implementation, v0.1
Date: 2026-07-27
Implementation target: new `rescue_catalog` crate

## Responsibility

AssetInventory scans only explicitly selected local directory roots and creates
an immutable observation snapshot of recognized audio files.

The module also supports `AssetFileSnapshotRequest v0.1` for a bounded list of
exact audio paths already observed as existing regular files. This operation
does not walk parent directories and does not search for alternatives. It
produces the existing `AssetInventoryResult v0.1` contract.

```text
AssetInventoryRequest v0.1 -> AssetInventoryResult v0.1
```

It deliberately separates:

```text
FileOccurrence = one file at one observed path
ContentRecord = one full SHA-256 + size identity, possibly at many paths
```

## Input

`AssetInventoryRequest` contains a caller-provided scan run ID, one or more
absolute roots and a positive entry limit. Roots are not canonicalized. A root
symlink or non-directory is rejected. Overlapping roots do not create duplicate
records for the same lexical path.

Recognized v0.1 extensions:

```text
wav aif aiff flac mp3 ogg m4a aac
```

Extension matching is ASCII case-insensitive. Format validity is not decoded in
this module.

## Output

`AssetInventoryResult` contains metadata, sorted `FileOccurrence` records,
content records, warnings and errors. Scan status is:

```text
complete
partial
failed
```

A complete result means all readable entries under accepted roots were visited
within the configured limit. It does not mean the whole computer was scanned.

## Hash And Race Policy

Every accepted regular file is streamed through full SHA-256. Metadata is
checked before opening, on the open handle and after hashing. On Unix, device
and inode identity must remain stable. An unstable file is excluded and
reported. SHA-256 proves byte identity between observed files; it does not prove
that a missing ALS requirement expected those bytes.

## Safety

```text
read-only scan
never follow file or directory symlinks
iterative traversal
positive entry limit
no canonicalization outside selected roots
no writes, copies, deletes or ALS parsing
no matching or package decision
```

## Errors

```text
INVENTORY_EMPTY_SCOPE
INVENTORY_INVALID_ENTRY_LIMIT
INVENTORY_ROOT_NOT_ABSOLUTE
INVENTORY_ROOT_NOT_FOUND
INVENTORY_ROOT_IS_SYMLINK
INVENTORY_ROOT_NOT_DIRECTORY
INVENTORY_ROOT_METADATA_FAILED
```

Entry-level read/hash failures are warnings and make the scan partial.

An exact-file snapshot fails closed if a requested path is relative, missing,
a symlink, not a recognized audio file, or changes while hashing. It never
silently drops one requested file and calls the remaining snapshot complete.

## Acceptance

```text
recognized audio files are hashed and recorded
non-audio files are ignored
identical bytes share one ContentRecord but retain separate occurrences
symlinks are reported and never followed
relative or invalid roots fail closed
entry limit produces a partial scan
ordering and identifiers are deterministic for one unchanged filesystem state
the scan is read-only
tests, clippy and module guard pass
```

## Known Limits

The MVP keeps the snapshot in memory and JSON. Incremental SQLite persistence,
OS-native cancellation/progress, cloud placeholders, Windows file identity
hardening and full-disk permission UX are later modules or adapters.
