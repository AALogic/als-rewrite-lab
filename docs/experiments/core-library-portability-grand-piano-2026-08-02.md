# Core Library Portability: Grand Piano

Status: narrow macOS Core Library default policy implemented; cross-platform portability research remains open

Date: 2026-08-02

## Observation

A Desktop Alpha report for a real copied ALS contained:

```text
304 reference occurrences
148 required unique audio assets
148 assets observed at their recorded paths
120 Grand Piano samples under the Ableton Live 11 Core Library
```

The 120 Grand Piano files form a multisample instrument rather than 120
independent user samples. The observed pattern is 30 pitches with four velocity
layers (`p`, `mf`, `f`, `ff`). This explains why a single native instrument can
dominate the flat dependency count.

## Important Distinction

An Ableton-native device and the audio content used by that device are not the
same dependency:

```text
native device
  device implementation and state stored or resolved by Live

native content
  Core Library or Pack audio files used by the device or preset
```

The device may be present while the required content is absent, belongs to a
different Live edition, is not installed, or is incompatible with the target
Live version.

## Existing Product Rule

The current `PRODUCT_SPEC.md` rule remains unchanged:

```text
classify Core Library and Factory Pack files as system_dependency
do not copy them by default
mark them portable_risk when the target environment is unknown
```

This rule is suitable for local organization, but it is not yet sufficient to
promise portability to another computer. Ableton's Collect All and Save can
collect Core Library and Pack files, and multisample instruments can produce
large packages. Live Set and Pack compatibility also depends on Live version
and installed content.

Official references:

- https://help.ableton.com/hc/en-us/articles/360000841004-Backward-Compatibility
- https://help.ableton.com/hc/en-us/articles/115000915804-Saving-Projects
- https://help.ableton.com/hc/en-us/articles/209071909-Transferring-Projects-to-another-computer
- https://cdn-resources.ableton.com/resources/pdfs/live-manual/12/2025-08-21/live12-manual-en.pdf

## Provisional Product Policy

Future packaging should distinguish three user intentions:

```text
local organization
  report and group system dependencies; do not collect them

move to a verified compatible environment
  omit only system content confirmed available in the target environment

strict portable archive or transfer
  offer collection of supported Core Library and Pack audio, with size impact
```

Until target compatibility is proven, a skipped dependency must remain visible
as `portable_risk`. The product must not claim that `found on this computer`
means `available on the target computer`.

Core Library and Pack dependencies should be grouped in the UI, for example:

```text
Ableton Core Library / Grand Piano
120 required samples
available locally
target portability unknown
```

The individual references may remain available in an expanded diagnostic view.

## Current Implementation Boundary

Confirmed macOS Core Library references using the observed type 5 pattern are
now recorded as `system_dependency`. The default current-path package leaves
them system-managed, creates no copy or rewrite operation, records
`portable_risk`, and validates that their ALS references remain unchanged.
Classification requires the complete evidence described in ADR-008; type 5 by
itself is insufficient.

## Required Experiments Before Automation

Use copied projects and never modify source ALS or source content.

1. Open an ALS-only Grand Piano project on another macOS installation with the
   same Live major version and edition.
2. Test Live 11 to Live 12 with the corresponding content installed.
3. Test the project where the target edition or required Pack is absent.
4. Test macOS to Windows with the same compatible Live content and determine
   whether Live resolves system content despite different native paths.
5. Compare Collect All and Save output with Core Library and Pack collection
   enabled and disabled.
6. Inspect how ALS references differ before and after collection.
7. Determine which stable identifiers, Pack metadata or path semantics can
   prove target availability without relying on an absolute source path.

## Decision Gate

Cross-platform omission, target compatibility claims and strict-portable
collection remain blocked until the experiments define:

```text
target environment contract
reliable Core Library / Pack classification
compatibility decision table
copy policy for each packaging mode
manifest fields for omitted system dependencies
UI warning and grouping behavior
```
