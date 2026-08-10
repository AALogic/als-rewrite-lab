# Plan 022: ProjectCatalogStore

Status: completed
Date: 2026-08-03

1. Add exact public store, load, metadata, stored-record and error contracts.
2. Validate module 021 snapshots and existing persisted state before writes.
3. Implement deterministic complete and partial freshness merge policies.
4. Add deterministic JSON serialization behind the storage adapter.
5. Implement create/replace through a synced temporary sibling on Unix and
   Windows without exposing paths in diagnostics.
6. Verify post-write round trip and idempotent replay.
7. Test corruption, partial coverage, stale records and preservation of prior
   valid bytes on failures.
8. Run module and full-workspace quality gates.
