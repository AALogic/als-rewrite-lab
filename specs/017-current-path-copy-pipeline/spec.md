# Module Specification: 017 CurrentPathCopyPipeline

Status: implementation update approved, v0.5
Date: 2026-08-03

## Purpose

CurrentPathCopyPipeline creates one safe Ableton Project copy using only audio
that still exists at paths recorded by the selected ALS. It never searches the
computer and never substitutes a moved candidate.

## Input

`CurrentPathCopyRequest v0.3` contains a non-empty run ID, an explicit rewrite
policy, one absolute regular
non-symlink source ALS, and fresh isolated staging, final target and private
ledger paths. Optional expected source ALS SHA-256 and expected
`PlanFingerprint v0.1` values bind execution to a previous read-only preview.

## Flow

```text
read and assess one ALS
-> create metadata-only bindings for existing safe recorded audio paths
-> separate confirmed Ableton Core Library system dependencies
-> plan current_paths_copy
-> stage ALS, directories and available audio
-> rewrite only supported available references
-> prove omitted references stayed unchanged through semantic diff
-> validate, write manifests and promote
```

Missing audio is reported and retained as a non-blocking omission. No search,
ranking of substitutes or candidate selection runs.

The flow does not invoke content-addressed AssetInventory and does not compute
SHA-256 for audio. `CurrentPathBindingResult v0.1` records current path and
observed size without claiming historical identity. ALS and small evidence
artifacts retain their hash checks.

## Output

`CurrentPathCopyResult v0.6` returns stage evidence, including the applied
rewrite policy and exact
system dependency count and current `PlanFingerprint v0.1`, and one of:

```text
complete_copy_ready_for_manual_check
incomplete_copy_ready_for_manual_check
complete_copy_preview_ready
incomplete_copy_preview_ready
rejected_before_read
read_stage_failed
snapshot_or_plan_blocked
preview_plan_changed
write_pipeline_failed
```

Complete means every user-managed supported audio asset was copied and
rewritten. Confirmed system dependencies remain managed by Ableton and do not
make the package incomplete.
Incomplete means the package is structurally valid but one or more requirements
remain linked to an original or missing location.

## Safety

- Original ALS and audio are read-only.
- Missing references are never edited.
- No output path may exist before the run.
- No output may be inside the source Ableton Project.
- No collision, stale source, unsafe path or unexpected semantic diff is tolerated.
- Incomplete is a successful copy outcome, not a portability claim.
- Every omission is recorded in the private ledger and portable manifest.
- Current-path audio uses `stable_source_and_size`; the portable manifest says
  explicitly that content identity was not computed.
- A supplied expected plan fingerprint is compared after read-only planning and
  before staging. Any semantic plan drift requires a new preview.
- A platform outside the Unix or bounded Windows x64 write profiles returns
  `write_pipeline_failed` with public error code `PIPELINE_REWRITE_FAILED`,
  preserves all sources and never promotes the staging directory.
- Same-path, same-size audio replacement remains an accepted current-path MVP
  limit and is not presented as historical content identity.

## Does Not Do

No folder scan, whole-computer index, substitute matching, batch processing,
cleanup, deletion, plugin portability or Ableton launch.

## Acceptance

- all available supported references produce a complete copy;
- safe Type 3 project-local references preserve their Samples/... locations and
  change only absolute Path values;
- existing unsupported non-system references block before staging and are never
  copied as unreferenced package files;
- confirmed Core Library dependencies create no copy or rewrite, do not block,
  and remain visible as `portable_risk`;
- one missing reference produces an incomplete copy and does not block;
- available references are copied and rewritten when another reference is missing;
- missing FileRef values remain unchanged;
- a changed source, collision, unsafe target or unsupported document blocks;
- original files remain unchanged;
- manifests identify complete versus incomplete output.
- an end-to-end test proves that audio hashes stay absent from plan, staging,
  validation, manifest and promotion records.
- appearing, disappearing or resized audio after preview changes the plan
  fingerprint and blocks before every write.
- equivalent operation ordering produces the same fingerprint.
- an unsupported atomic-replacement platform fails closed without publishing a
  target.
- a native Windows synthetic run proves the complete pipeline on local NTFS,
  including rewrite, manifests, promotion, repeat verification, and unchanged
  source files.
