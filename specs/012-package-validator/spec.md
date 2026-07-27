# Module Specification: 012 Validator / SemanticDiff

## Purpose

Validator independently decides whether a completed staging directory is safe
to promote. It consumes `PackagePlan v0.1`, `StagingExecutionResult v0.1`,
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

`PackageValidationResult v0.1` contains one file record per copy operation,
one semantic record per rewrite operation, structured errors, and either
`validation_passed` or `validation_failed`.

## File Validation

1. Final target must still be absent.
2. Staging must be a real non-symlink directory.
3. Its regular files must equal the planned target set exactly.
4. Symlinks and unexpected files are rejected.
5. Every staged audio file must match planned full SHA-256 and size.
6. Rewritten ALS must match the hash returned by ALSRewriter.
7. Original ALS must still match the source snapshot hash.

## Semantic Diff

1. Decompress source and rewritten ALS with bounded limits.
2. Parse both documents and require the Ableton root.
3. Resolve every approved `SampleRef[n]/FileRef`.
4. Require source values to equal planned old values and output values to equal
   planned new values.
5. Replace only those six compared value ranges with identical sentinel tokens.
6. Require the remaining decompressed XML to be byte-for-byte identical.

This proves that historical references, unrelated active references, metadata,
devices, and formatting did not change.

## Safety Boundary

The module is read-only. It never fixes, deletes, promotes, rewrites, or writes
a manifest. Repeated validation of unchanged input is deterministic and leaves
all files unchanged.

## Errors

Includes `ORIGINAL_FILE_CHANGED`, `VALIDATION_FILE_HASH_MISMATCH`,
`VALIDATION_UNEXPECTED_STAGED_FILE`, `VALIDATION_STAGING_SYMLINK`,
`VALIDATION_FINAL_TARGET_EXISTS`, `SEMANTIC_DIFF_VALUE_MISMATCH`, and
`SEMANTIC_DIFF_UNEXPECTED_CHANGE`.

## Acceptance Criteria

Tests cover a passing end-to-end staging/rewrite fixture, tampered audio,
unexpected files, unrelated XML changes, historical-reference changes,
approved-field mismatch, original-source mutation, existing final target,
cross-run contract mismatch, and read-only repeatability.

## Does Not Do

No repair, matching, planning, copying, rewrite, manifest writing, package
promotion, Ableton runtime launch, or user acceptance.
