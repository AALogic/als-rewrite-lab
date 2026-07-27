# Module Specification: 011 ALSRewriter

## Purpose

ALSRewriter applies an accepted, snapshot-bound `RewriteOperation v0.1` list
to the ALS copy inside completed staging. It implements only the E-03
`live11_3_external_to_imported_v0.1-experimental` laboratory profile.

## Gate Status

CONFIRMED for the structural rewrite mechanism and EXPERIMENTAL_LAB_ONLY for
Ableton Live 11.3 external type-1 references. No production or cross-version
support is claimed.

## Inputs

- `ALSRewriteRequest` identifying the exact staging root.
- A complete `PackagePlan v0.1` in laboratory rewrite mode.
- A successful `StagingExecutionResult v0.1` for the same plan and ALS hash.
- Rewrite operations containing source hash, `SampleRef[n]/FileRef` locator,
  approved old values, approved new values, and exactly three fields:
  `Path`, `RelativePath`, and `RelativePathType`.

## Outputs

`ALSRewriteResult v0.1` records source and rewritten compressed SHA-256,
per-operation status, ruleset, warnings, errors, and the staged ALS location.

## Algorithm

1. Reject mismatched plan/staging/request contracts before reading or writing.
2. Recalculate the staged ALS compressed SHA-256 and require the exact source
   snapshot hash from the plan.
3. Decompress with the ALSReader limits and parse XML.
4. Enumerate `SampleRef` exactly as ALSReader does.
5. Resolve only the direct `FileRef` named by the approved locator.
6. Require every current old value to equal the operation snapshot.
7. Use roxmltree byte ranges for the three `Value` attributes.
8. XML-escape replacements and apply non-overlapping ranges from last to first.
9. Parse the result and verify all three new values at every locator.
10. Encode deterministic gzip bytes to a create-new temporary sibling.
11. Validate gzip, UTF-8, XML, and Ableton root before replacing only the
    staged ALS.

## Safety Boundary

- The original source ALS is never opened for writing.
- Global string replacement is forbidden.
- Historical `OriginalFileRef`, unrelated active refs, metadata, devices, and
  all XML outside approved attribute value ranges remain byte-preserved.
- Any snapshot, locator, old-value, ruleset, target, or staging mismatch fails
  closed before staged replacement.
- A second run against an already rewritten staged ALS returns an explicit
  snapshot mismatch instead of stacking rewrites.

## Does Not Do

No asset discovery, matching, copy, final package promotion, deletion, repair,
heuristic locator recovery, version guessing, or user verification.

## Errors

Structured errors include `REWRITE_PLAN_NOT_READY`,
`REWRITE_STAGING_NOT_COMPLETE`, `REWRITE_SOURCE_HASH_MISMATCH`,
`REWRITE_LOCATOR_MISMATCH`, `REWRITE_OLD_VALUE_MISMATCH`,
`REWRITE_RULE_UNSUPPORTED`, `REWRITE_TARGET_CONTRACT_MISMATCH`,
`REWRITE_OUTPUT_XML_INVALID`, and atomic-write failures.

## Acceptance Criteria

Tests prove targeted three-field change, historical/unrelated preservation,
snapshot mismatch rejection, old-value mismatch rejection, locator rejection,
ruleset rejection, duplicate-reference rejection, repeat safety, and XML
escaping on Unix. Non-Unix tests prove that the unsupported atomic replacement
adapter fails closed and preserves the staged ALS.

## Known Limits

Atomic replacement of the staged ALS is currently enabled only on Unix. Other
platforms fail closed at the isolated I/O adapter until an equivalent
replace-existing primitive is implemented and tested. Live 9/10/12, type 5,
same-name collisions, Windows path semantics, and unknown XML contexts remain
unsupported.
