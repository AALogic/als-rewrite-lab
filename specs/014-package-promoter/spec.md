# Module Specification: 014 PackagePromoter

## Purpose

PackagePromoter is the only module that turns a validated staging directory
into the final project directory. It accepts no partial state and makes no
content decisions.

## Gate Status

CONFIRMED for a same-filesystem laboratory promotion to an absent target.

## Inputs

- `PackagePromotionRequest` with a promotion ID.
- The same successful `PackagePlan v0.1`, staging, rewrite, validation, and
  manifest-write results from one pipeline run.
- A package manifest inside staging and a private ledger outside it.

## Preconditions

1. All prior statuses are successful and their run IDs agree.
2. Original ALS still has the planned SHA-256.
3. Private ledger still has the recorded hash and size.
4. Staging contains exactly every validated package file plus the portable
   manifest, with matching full hashes and sizes.
5. No symlink or unexpected file exists in the package.
6. Final target parent already exists and is a non-symlink directory.
7. Final target is absent.
8. On Unix, staging and target parent are on the same filesystem.

## Operation

After preflight, rename the entire staging directory to the final target. Then
re-run exact package verification and sync the target parent. If post-check or
sync fails, attempt to rename the directory back to staging and report whether
rollback succeeded.

## Idempotency

If staging is absent and the final target exists, the module does not rewrite
or move it. It verifies the complete package against the same evidence and
returns `already_promoted_verified` only when every byte still matches.

## Safety Boundary

An existing final target is never intentionally overwritten, merged, cleaned,
or deleted. Source files and private ledger are read-only. No recursive delete
exists. This module does not claim hardened protection against a malicious
local process racing the final existence check; a platform-specific
no-replace directory primitive remains required before commercial release.

## Outputs

`PackagePromotionResult v0.1` records final status, all verified files,
pipeline identities, final root, manual-check readiness, and structured errors.
Successful promotion sets manual-check status to
`ready_for_manual_ableton_check`; it is not yet user verification.

## Acceptance Criteria

Tests prove successful move without source mutation, existing-target
protection, tamper and unexpected-file rejection, ledger gate, status gate,
idempotent repeat verification, symlink rejection, and original-source change
detection.

## Does Not Do

No planning, copying of individual files, rewrite, manifest construction,
deletion, cleanup, Ableton launch, or user acceptance.
