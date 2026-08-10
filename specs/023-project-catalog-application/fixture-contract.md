# Fixture Contract 023: ProjectCatalogApplicationService

Status: verified
Date: 2026-08-03

Tests use temporary scan/store roots with invalid ALS bytes because catalog
refresh must never parse ALS content.

```text
Project A/
  Ableton Project Info/
  Main.als
  Alternate.als
  Backup/Old.als

Duplicate root A/Project/Main.als
Duplicate root B/Project/Main.als
Loose/orphan.als
Outer/Ableton Project Info/
Outer/Inner/Ableton Project Info/
Outer/Inner/Ambiguous.als
```

Sequential complete and partial refreshes produce observed, retained and stale
records. Selection fixtures cover exact revision, stale revision, unknown ID,
stale record, duplicate ID/path and one manual absolute ALS path.

Rust-to-JSON fixtures are consumed by the TypeScript contract test. UI tests use
the same projected contract and never synthesize grouping policy.
