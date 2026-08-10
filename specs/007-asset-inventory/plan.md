# Plan 007: AssetInventory

Status: ready
Date: 2026-07-27

1. Add a catalog crate with explicit inventory contracts.
2. Validate absolute, non-symlink directory roots and the scan limit.
3. Traverse iteratively without following symlinks.
4. Stream full SHA-256 with before/open/after stability checks.
5. Sort file occurrences and derive deterministic content groups.
6. Test scope, limits, duplicates, read-only safety and downstream handoff.
7. Run workspace quality commands and the module guard.
