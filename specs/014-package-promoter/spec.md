# Module Specification: 014 PackagePromoter

## Purpose

PackagePromoter is the only module that turns a validated staging directory
into the final project directory. It accepts no partial state and makes no
content decisions.

## Gate Status

CONFIRMED for a same-filesystem laboratory promotion to an absent target.

## Inputs

- `PackagePromotionRequest` with a promotion ID.
- The same successful `PackagePlan v0.4`, staging, rewrite, validation, and
  manifest-write results from one pipeline run.
- A package manifest inside staging and a private ledger outside it.

## Preconditions

1. All prior statuses are successful and their run IDs agree.
2. Original ALS still has the planned SHA-256.
3. Private ledger still has the recorded hash and size.
4. Staging contains exactly every validated package file plus the portable
   manifest. Hash-backed files match full hashes and sizes; metadata-only audio
   matches regular-file type and size without another byte read.
5. No symlink or unexpected file exists in the package.
6. Final target parent already exists and is a non-symlink directory.
7. Final target is absent.
8. Staging and target parent are on the same filesystem or local NTFS volume.
9. Windows Alpha roots are drive-letter paths below `MAX_PATH` and no existing
   path component is a reparse point.

## Operation

After preflight, rename the entire staging directory to the final target. Unix
uses its native rename; Windows uses `MoveFileExW` without replacement flags so
an existing target cannot be overwritten. Then re-run exact package
verification and sync or write-through according to platform capability. If
post-check fails, attempt the same no-clobber move back to staging and report
whether rollback succeeded.

## Idempotency

If staging is absent and the final target exists, the module does not rewrite
or move it. It verifies the complete package against the same policy-specific evidence and
returns `already_promoted_verified` only when every byte still matches.

## Safety Boundary

An existing final target is never intentionally overwritten, merged, cleaned,
or deleted. Source files and private ledger are read-only. No recursive delete
exists. The Windows adapter uses a no-replace primitive. Unix still does not
claim hardened protection against a malicious local process racing the final
existence check; that pre-release hardening remains separate.

## Outputs

`PackagePromotionResult v0.2` records final status, all verified files,
pipeline identities, final root, manual-check readiness, and structured errors.
Successful promotion sets manual-check status to
`ready_for_manual_ableton_check`; it is not yet user verification.

## Acceptance Criteria

Tests prove successful move without source mutation, existing-target
protection, tamper and unexpected-file rejection, ledger gate, status gate,
idempotent repeat verification, symlink rejection, and original-source change
detection. Native Windows tests also prove Unicode paths, target collision,
same-volume local NTFS enforcement, reparse-point rejection, and repeated
verification.

## Does Not Do

No planning, copying of individual files, rewrite, manifest construction,
deletion, cleanup, Ableton launch, or user acceptance.
