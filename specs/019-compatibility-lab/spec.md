# Module Specification: 019 Compatibility Lab

Status: approved for implementation
Date: 2026-08-03

## Parent Product Capability

Compatibility Lab supports evidence-driven expansion of safe ALS rewrite
profiles without weakening the strict Desktop Alpha.

## Supported Use Case

A private tester selects a copied or original read-only ALS from an
unconfirmed Ableton version, inspects its dependencies, explicitly requests an
experimental copy, manually opens the generated copy in Ableton, and copies a
redacted report for engineering review.

## Responsibility

Compatibility Lab:

- describes the rewrite-relevant structure of active audio references without
  exposing paths or filenames;
- distinguishes confirmed Live 11.3 profile evidence from an unconfirmed
  document with known reference shapes;
- authorizes experimental current-path planning only in a compile-time lab
  build and only with explicit user consent;
- assembles a versioned `CompatibilityTestReport v0.1` from machine facts and a
  manual Ableton outcome.

## Non-Responsibilities

It does not:

- declare Live 10, Live 11.2 or any other version officially supported;
- infer sample identity from version metadata;
- permit unknown reference shapes;
- search for missing samples;
- bypass staging, semantic diff, package validation or promotion rules;
- modify original ALS or audio;
- upload reports or files.

## Contracts

`RewriteCompatibilityAssessment v0.1` contains redacted document profile,
overall evidence status, aggregate counts and one
`RewriteReferenceCompatibility v0.1` per active audio reference. Reference
records contain IDs, context, RelativePathType, strict/lab support booleans and
reason codes, but no path or filename values.

`DesktopApplicationProfile v0.1` reports whether the running binary is
`strict_alpha` or `compatibility_lab`.

`DesktopPrepareCopyRequest v0.2` carries `experimental_compatibility_consent`.
The application rejects consent in a binary without the compile-time lab
feature. `CurrentPathCopyRequest v0.3` carries the selected rewrite policy to
the planner. Preview and execution remain source- and plan-bound.

`DesktopDiagnosticReport v0.3` includes the structural compatibility
assessment. `DesktopCopyDiagnosticReport v0.2` includes rewrite policy, safe
document metadata, compatibility status and operation duration.

`CompatibilityTestReportRequest v0.1` contains the two machine reports,
manual outcome and optional tested Ableton version. The application returns
`CompatibilityTestReport v0.1` with one provisional conclusion:

```text
candidate_success
candidate_failure
insufficient_evidence
```

## Structural Policy

An unconfirmed reference is experimentally compatible only when all are true:

- usage context is `audio_clip` or `simpler_multisample`;
- locator is the exact direct `SampleRef[n]/FileRef` form;
- Type 1 has a safe single-component filename and uses the existing Imported
  three-field rewrite;
- or Type 3 has a safe `Samples/...` relative path and uses the existing
  Path-only relocation;
- the normal planner can build an explicit rewrite operation.

Any selected reference outside those profiles becomes a blocking unresolved
requirement. Version metadata alone never changes that result.

## Safety Invariants

- Strict builds cannot accept experimental compatibility consent.
- Strict `current_paths_copy` behavior remains unchanged.
- Compatibility Lab writes only to fresh staging and a user-selected absent
  target.
- Original ALS and audio remain read-only.
- Unknown structures block before staging.
- Reports contain no absolute paths, filenames, raw XML or host identity.
- Manual success is evidence, not automatic support promotion.

## Acceptance Criteria

- a strict build still blocks a structurally identical Live 10/11.2-labelled
  fixture;
- a lab build can preview and execute the same fixture after explicit consent;
- a lab build blocks an unknown context, unsafe Type 3 path and unknown path
  type before write;
- source ALS and audio are byte-identical after every test;
- report copying works after analysis, blocked preview and completed copy;
- manual outcomes produce the correct provisional conclusion;
- strict and lab installers have different names and identifiers.

## Gate Status Summary

The feature is authorized as a private evidence-gathering build. Compatibility
with real Live 10 and Live 11.2 ALS remains unconfirmed until reports and manual
Ableton-open results are reviewed.
