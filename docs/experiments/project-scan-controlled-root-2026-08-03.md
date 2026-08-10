# Project Scan Controlled-Root Evidence, 2026-08-03

Status: completed local read-only evidence
Module: 020 ALSProjectScanner v0.1
Scope: one private root containing copied Ableton Projects and laboratory output

## Privacy Boundary

The source root, Project names, Set names and native paths remain private and
are not recorded in this document. The experiment output contained aggregate
counts only.

## Method

1. Compute one aggregate SHA-256 over the ordered SHA-256 list of all ALS files
   under the controlled root. This hash was produced by the experiment harness,
   not by ALSProjectScanner.
2. Run ALSProjectScanner with:

```text
one explicit approved root
no explicit exclusions
max_entries = 100000
max_depth = unlimited
traversal_policy = project_scan_v0.1
```

3. Record only aggregate scanner metadata and elapsed wall time.
4. Recompute the aggregate ALS hash after the scan.

## Result

```text
scan_status: complete
directories_visited: 916
entries_visited: 10457
ALS files observed: 36
exact Ableton Project Info markers observed: 3
symlinks skipped: 0
warnings: 0
errors: 0
elapsed wall time, warm local run: 180 ms
aggregate ALS hash before:
  04c7b121776e52f703c1b7e8a415c6d83e1ade6d93be340d60a74d6147a7d6dd
aggregate ALS hash after:
  04c7b121776e52f703c1b7e8a415c6d83e1ade6d93be340d60a74d6147a7d6dd
```

## Interpretation

The scanner completed the declared root without warnings or errors, discovered
both Set and structural marker evidence, and left ALS bytes unchanged. This is
native macOS evidence for the read-only, lightweight traversal behavior.

It does not prove:

```text
Windows junction/reparse behavior
macOS protected-directory permission handling
whole-computer root selection quality
ProjectFolder / BackupSet grouping
SQLite persistence
Desktop progress delivery
batch copy behavior
```

Those facts remain owned by later modules or platform experiments.
