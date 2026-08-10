# Module Specification: 012 Validator / SemanticDiff

## Purpose

Validator independently decides whether a completed staging directory is safe
to promote. It consumes `PackagePlan v0.5`, `StagingExecutionResult v0.3`,
and `ALSRewriteResult v0.1`, but trusts none of their success claims without
checking the filesystem and both ALS snapshots again.

## Gate Status

CONFIRMED for bounded laboratory validation.

## Inputs

- A validation request identifying the same absolute staging root.
- One complete laboratory package plan.
- Successful staging and rewrite results tied to the same plan, execution, and
  source ALS hash.
- The unchanged source ALS and current staged package.

## Outputs

`PackageValidationResult v0.4` contains one directory record per planned
directory, one file record per copy operation, one semantic record per rewrite
operation, structured errors, and either
`validation_passed` or `validation_failed`.

## File Validation

1. Final target must still be absent.
2. Staging must be a real non-symlink directory.
3. `Ableton Project Info` must be a planned, real non-symlink directory.
4. Every other planned directory must exist as a real non-symlink directory.
5. Its regular files must equal the planned target set exactly.
6. Symlinks and unexpected files are rejected.
7. Hash-backed files must match planned SHA-256 and size. The metadata-only
   current-path audio must be a regular non-symlink file with the planned size;
   validation does not read its bytes again.
8. Rewritten ALS must match the hash returned by ALSRewriter.
9. Original ALS must still match the source snapshot hash.
10. Every system dependency disposition must be unique, must declare
    `leave_system_managed/portable_risk`, and must not overlap any copy source
    or rewrite reference.

## Semantic Diff

1. Decompress source and rewritten ALS with bounded limits.
2. Parse both documents and require the Ableton root.
3. Resolve every approved `SampleRef[n]/FileRef`.
4. Require source values to equal planned old values and output values to equal
   planned new values.
5. Replace only rule-approved changed value ranges with identical sentinel
   tokens in each document.
6. Require the remaining decompressed XML to be byte-for-byte identical.

This proves that historical references, unrelated active references, metadata,
devices, and formatting did not change.

## Safety Boundary

The module is read-only. It never fixes, deletes, promotes, rewrites, or writes
a manifest. Repeated validation of unchanged input is deterministic and leaves
all files unchanged.

## Errors

Includes `ORIGINAL_FILE_CHANGED`, `VALIDATION_FILE_HASH_MISMATCH`,
`VALIDATION_FILE_SIZE_MISMATCH`,
`VALIDATION_UNEXPECTED_STAGED_FILE`, `VALIDATION_STAGING_SYMLINK`,
`VALIDATION_FINAL_TARGET_EXISTS`, `SEMANTIC_DIFF_VALUE_MISMATCH`, and
`SEMANTIC_DIFF_UNEXPECTED_CHANGE`.

## Acceptance Criteria

Tests cover a passing end-to-end staging/rewrite fixture, tampered audio,
unexpected files, unrelated XML changes, historical-reference changes,
approved-field mismatch, original-source mutation, existing final target,
cross-run contract mismatch, Type 3 Path-only semantic diff, and read-only
repeatability. A dedicated negative test proves that a system dependency cannot
be copied or rewritten.

## Does Not Do

No repair, matching, planning, copying, rewrite, manifest writing, package
promotion, Ableton runtime launch, or user acceptance.
