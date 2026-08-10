# Fixture Contract 021: ProjectCatalogBuilder

Status: ready
Date: 2026-08-03

Tests construct `ProjectScanResult v0.1` values directly. No test touches the
filesystem.

## Required Logical Fixture

```text
root-a/
  Project A/
    Ableton Project Info/
    Main.als
    Alternate.als
    Backup/
      Main [timestamp].als
  Standalone/
    orphan.als

root-b/
  Project A/
    Ableton Project Info/
    Main.als

nested markers:
  Outer/Ableton Project Info/
  Outer/Inner/Ableton Project Info/
  Outer/Inner/Ambiguous.als

marker-only:
  Empty Project/Ableton Project Info/
```

The fixture builder assigns valid scanner metadata and deterministic synthetic
observation IDs. Individual tests alter one fact at a time.

## Required Cases

```text
standard one-folder one-Set association
multiple main Sets
exact Backup component
ProjectFolder named Backup and directory named Backups are not backups
standalone ungrouped Set
same names under different roots
nested ambiguous marker candidates
marker with no associated Set
partial source scan
cancelled source scan
failed source scan
inconsistent counts
duplicate observation ID
duplicate native path
unsupported scanner version
deterministic repeated build
moved native path changes local occurrence ID
path-free catalog warnings
fake ProjectCatalogStore consumer
```

No fixture may contain a primary Set, version-family or ProjectWork field.
