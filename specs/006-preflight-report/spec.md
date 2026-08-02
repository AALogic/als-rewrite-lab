# Module Spec 006: PreflightReport

Status: implemented, v0.2
Date: 2026-07-27
Implementation target: `rescue_analyzer` and read-only `rescue preflight`

## Responsibility

PreflightReport turns confirmed project-structure evidence and a dependency
assessment into a local, user-readable report for one selected ALS.

```text
ProjectDiscoveryResult v0.1 + DependencyAssessmentResult v0.2
-> PreflightReport v0.2
```

It reports requirements with observed candidates, requirements needing search,
unknowns and risks. It does not call a requirement resolved merely because a
path exists.

Permanent rule: candidate availability is not asset resolution.

## Product Traceability

This completes the first read-only vertical slice:

```text
selected ALS -> discover root -> read -> extract -> observe -> assess -> report
```

## Input And Validation

Both inputs must be free of fatal errors. Their selected ALS paths must agree.
An untrusted input produces a blocked report with a structured error and no
requirement details.

## Output

`PreflightReport v0.2` contains report metadata, project context, aggregate
summary, one presentation record per `RequiredAsset`, notices and errors.

Confirmed system dependencies are reported separately from missing assets.
Their requirement records include source category, management class and
portability status.

Overall statuses:

```text
candidates_observed_not_resolved
needs_asset_search
needs_review
blocked
```

The report may contain local paths because it is a private local report. It is
not the portable `PackageManifest`, which must redact local provenance later.

## Safety And Boundaries

```text
pure report projection
no filesystem access
no matching or selection
no copy/rewrite/delete
no claim of Ableton runtime success
```

## Errors

```text
PREFLIGHT_UNTRUSTED_PROJECT_DISCOVERY
PREFLIGHT_UNTRUSTED_ASSESSMENT
PREFLIGHT_SOURCE_MISMATCH
```

## Acceptance

```text
counts exactly mirror the assessment
missing requirements request asset search
unknown requirements request review
all observed candidates remain unresolved
project evidence and risks are explainable
output is deterministic and read-only
CLI prints the report as JSON
tests, clippy and module guard pass
```
