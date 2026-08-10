# Module Specification: 015 LaboratoryPipeline

## Purpose

LaboratoryPipeline composes the confirmed, single-project modules into one
bounded vertical slice. It proves that their public contracts can carry one
project from read-only discovery through a promoted rescue copy without
modifying source files.

## Gate Status

READY for the narrow Live 11.3 `AudioClip` laboratory profile after policy v0.3
selection validation. Other versions and usage contexts remain blocked.

## Inputs

`LaboratoryPackageRequest v0.2` contains:

- a non-empty run ID;
- one existing, regular, non-symlink ALS source path;
- one or more explicitly selected, bounded, non-symlink scan roots;
- a finite scan-entry limit;
- an optional `UserSelectionSet v0.1`, already parsed at the CLI boundary;
- fresh and isolated staging, final-target, and private-ledger paths.

Every path is native `PathBuf` data. Platform-specific path interpretation
remains inside the modules that own it.

## Ordered Flow

1. Validate the complete request before reading or writing.
2. Discover and select exactly the requested ALS.
3. Read ALS and preserve its analysis snapshot.
4. Extract active audio dependencies.
5. Observe recorded paths without searching or mutating them.
6. Assess which audio assets are required.
7. Build a read-only preflight report.
8. Inventory only the explicitly selected roots.
9. Resolve inventory occurrences against required assets and validate any
   source-bound explicit user selections.
10. Build an immutable package plan.
11. Execute verified copies in a fresh staging directory.
12. Rewrite only approved, snapshot-bound FileRef fields in the staged ALS.
13. Validate files and semantic ALS differences independently.
14. Write a private ledger outside the package and a redacted portable
    manifest inside it.
15. Promote validated staging to an absent final target.

Each stage consumes only public contracts. The pipeline stops after the first
blocking or failed stage and retains all completed stage outputs in its result.
No write-capable module is called before the package plan is ready.

## Safety Boundary

- Source ALS and source audio are read-only throughout the flow.
- `/` and equivalent unbounded scan roots are rejected.
- Staging, final target, or private ledger paths that already exist are
  rejected before ALS reading.
- A dangling symlink or inaccessible output entry is treated as occupied.
- Output scopes must be isolated from the source and from one another.
- Project discovery must confirm exactly one structural Ableton Project root,
  and filesystem-resolved output parents, including aliases and case variants,
  must remain outside it.
- The pipeline never removes, merges, cleans, or overwrites user data.
- Ambiguous or unresolved dependencies block before staging.
- A moved candidate without explicit selection blocks before staging.
- A stale selection, changed selected file or selection for another ALS blocks.
- Unsupported rewrite evidence blocks before staging.
- A laboratory plan with zero approved rewrite operations blocks before staging.
- Successful static validation yields `ready_for_manual_ableton_check`; it is
  not proof that Ableton opened the result successfully.
- Unix and the bounded Windows x64 Alpha profile have implemented atomic
  staged-ALS replacement and run the supported write-path test.
- Any other platform without implemented atomic staged-ALS replacement returns
  `write_pipeline_failed` with `PIPELINE_REWRITE_FAILED`, preserves source
  files and never promotes staging to the final target.

## Outputs

`LaboratoryPackageResult v0.1` contains the run status, the last completed
stage, every available public stage result, and structured pipeline errors.
Absent outputs mean that their stages did not run; the pipeline does not
fabricate successful placeholders.

## Determinism And Retry

Given identical source bytes, scan content, paths, limits, rulesets, and fresh
output locations, planning and generated contents are deterministic. A failed
run never reuses a partial final target. A successful run may be inspected or
verified by downstream tooling, but a new laboratory run uses new output and
ledger paths.

## Acceptance Criteria

Tests prove that unconfirmed candidate identity, unknown Project roots, lexical
or aliased outputs inside a confirmed source Project, zero-reference plans,
missing or ambiguous samples, existing or symlink outputs, unbounded scans, and
unsupported Live versions all block before staging. Completed discovery
evidence remains in blocked results. Write modules retain their isolated tests,
but the composed pipeline does not claim a successful package until the missing
decision and rewrite-support contracts exist.
The supported-platform test proves the complete write path, while a separate
unsupported-platform test proves the same request fails closed.

## Does Not Do

No GUI, full-disk scan, batch processing, cleanup, deletion, VST or preset
portability, automatic Ableton launch, user acceptance, or production support
claim. It does not duplicate domain rules owned by the composed modules.
