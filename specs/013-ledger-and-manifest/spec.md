# Module Specification: 013 PrivateLedger / PackageManifest

## Purpose

This module records a validated packaging run in two deliberately different
artifacts:

- `PrivateLedger v0.1` is local audit evidence containing full paths, plans,
  operation results, errors, and validation facts.
- `PackageManifest v0.1` is placed in the portable project and contains only
  relative package paths, hashes, sizes, rewrite rules, and validation summary.

The separation prevents a handoff package from leaking the user's local folder
layout while retaining enough private evidence to diagnose a failed run.

## Gate Status

CONFIRMED for laboratory package evidence.

## Inputs

- `ManifestWriteRequest` with a private absolute ledger path outside staging
  and a safe relative package-manifest path.
- One `PackagePlan v0.1`, successful staging and rewrite results, and a
  `PackageValidationResult v0.1` with `validation_passed`.

## Outputs

`ManifestWriteResult v0.1` records both output paths, hashes, sizes, write
statuses, run identities, and structured errors.

## Portable Manifest Rules

- Include relative package paths, content SHA-256, byte sizes, file roles,
  approved XML locators/fields, ruleset version, and validation summary.
- Exclude source paths, staging root, final absolute root, old absolute sample
  paths, new absolute paths, and private diagnostics.
- Reject any serialized string that looks like a POSIX, UNC, or drive-letter
  absolute path.

## Write Rules

1. Build, serialize, and privacy-check both artifacts before writing.
2. The package manifest may create only its safe parent directory under staging.
3. The private-ledger parent must already exist.
4. Write to a create-new temporary sibling and sync it.
5. Create the final name by a no-clobber hard link.
6. Existing byte-identical output is accepted as `already_present_verified`.
7. Existing different content is a conflict and is never overwritten.

## Safety Boundary

No source or final project file is changed. A failed validation blocks both
artifacts. A private ledger is forbidden inside the portable package. The
writer makes no domain decisions and does not promote staging.

## Acceptance Criteria

Tests prove dual writes, portable redaction, private provenance, idempotent
repeat, no-clobber conflict behavior, validation gate, traversal rejection,
planned-file collision rejection, final-target gate, and no temporary residue.

## Does Not Do

No scan, matching, copy, ALS rewrite, package validation, final promotion,
cleanup, telemetry, upload, or Ableton runtime verification.
