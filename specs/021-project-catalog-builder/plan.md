# Plan 021: ProjectCatalogBuilder

Status: completed
Date: 2026-08-03

1. Add exact public catalog input/output contracts to `rescue_catalog`.
2. Validate upstream scanner version, status, counts, IDs and native paths.
3. Create deterministic local path-derived folder, Set and observation IDs.
4. Build one physical folder for every exact marker observation.
5. Associate each ALS with zero, one or many lexical marker roots without
   silently resolving ambiguity.
6. Classify exact `Backup` path components and populate main/backup folder lists.
7. Produce path-free warnings and propagate partial source coverage.
8. Test the pure transformation and a fake persistence consumer.
9. Run the full module and workspace verification sequence.
