# Module Specification: 010 StagingExecutor

## Purpose

StagingExecutor is the first filesystem-writing module. It executes only the
immutable `CopyOperation` list from an accepted `PackagePlan v0.1` inside a
new, isolated staging directory. It never chooses assets, rewrites ALS, writes
the final target, or touches source files.

## Gate Status

CONFIRMED for bounded local laboratory use.

## Inputs

- `StagingExecutionRequest` with an execution ID and a new absolute staging root.
- `PackagePlan v0.1` with status `ready_copy_only` or
  `ready_for_laboratory_execution`.
- Every planned source must be a regular non-symlink file with the planned
  byte length and full SHA-256.

## Outputs

`StagingExecutionResult v0.1` records every attempted operation, observed
hash and size, completion status, warnings, and structured errors.

## Behavior

1. Validate the whole plan before creating staging.
2. Reject unresolved requirements, plan errors, unsafe relative paths,
   duplicate targets, unsupported collision policy, an existing staging root,
   or staging equal to the final target.
3. Create staging only as a new directory.
4. Stream each source into a create-new temporary file while calculating
   SHA-256 and size.
5. Rename the temporary file to its staged relative target only after both
   values match the plan.
6. Stop on first failed copy and report `copy_failed`.
7. Return `staging_complete` only after every operation is verified.

## Safety Boundary

- Source paths are read-only.
- The final target root is not created or modified.
- Existing staging directories are never reused or cleaned.
- No overwrite mode exists; collision policy is `fail_if_exists`.
- A partial staging directory is evidence of failure, never a finished package.
- The module may remove only its own unpromoted temporary file.

## Does Not Do

No matching, policy decisions, rewrite, semantic validation, manifest writing,
package promotion, cleanup, full-disk scan, or UI behavior.

## Errors

`STAGING_PLAN_NOT_READY`, `STAGING_ROOT_UNSAFE`,
`STAGING_ROOT_EXISTS`, `STAGING_EQUALS_FINAL_TARGET`,
`STAGING_TARGET_PATH_UNSAFE`, `COPY_SOURCE_NOT_REGULAR_FILE`,
`COPY_SIZE_MISMATCH`, `COPY_HASH_MISMATCH`, and explicit I/O failures.

## Acceptance Criteria

Tests prove verified copy, hash mismatch rejection, no overwrite on repeat,
traversal rejection, symlink rejection, blocked-plan rejection, and that both
sources and the final target remain untouched.

## Known Limits

This v0.1 executor narrows race risk with pre-open and opened-handle metadata
checks, but it is not yet claimed as hardened against a malicious local process
that swaps a path between checks. That security-hardening item remains explicit
before a commercial release.

