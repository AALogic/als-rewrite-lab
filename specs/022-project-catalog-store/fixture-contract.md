# Fixture Contract 022: ProjectCatalogStore

Status: ready
Date: 2026-08-03

Tests use a private temporary directory and synthetic module 021 snapshots.
No real ALS, audio file or user catalog is read or modified.

## Logical Sequence

```text
snapshot complete-1
  Project A -> Main.als, Alternate.als, Backup/old.als
  Project B -> Remix.als

snapshot partial-2
  Project A -> Main.als with changed metadata
  Project B and other Project A Sets not observed

snapshot complete-3
  Project A -> Main.als
  Project B and other Sets not observed
```

Expected behavior:

```text
complete-1 stores all records as observed
partial-2 updates Main but retains all unseen records without stale claim
partial-2 cannot shrink Project A membership
complete-3 marks unseen records not_observed_in_latest_complete_scan
no record is physically deleted
complete scan under a changed coverage scope retains unseen prior records
```

Additional fixtures cover identical replay, corrupt JSON, unsupported schema,
failed input snapshot, target symlink where supported, deterministic bytes and
path-free diagnostics.
