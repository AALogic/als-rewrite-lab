# ADR-008: Core Library As A System Dependency

## Status

Accepted for the macOS desktop MVP on 2026-08-02.

## Context

Ableton Sets can contain active audio references to files inside the installed
Ableton Core Library. In the observed Live 11.3 corpus these references use
`RelativePathType = 5`. Treating them like user-managed audio makes the package
planner attempt an unsupported rewrite and blocks otherwise valid project
copies.

## Decision

`DependencyAssessment` may classify a reference as
`ableton_core_library/system_dependency` only when all confirmed macOS evidence
is present:

- `RelativePathType = 5`;
- a safe `Samples/...` relative path;
- an absolute path under an Ableton `.app/Contents/App-Resources/Core Library`;
- the exact recorded path is observed as a safe regular file without a size
  conflict.

The default package policy records `leave_system_managed`, performs no audio
copy and creates no ALS rewrite for that requirement. The dependency does not
count as missing and does not block execution. It is recorded as
`portable_risk` because the target computer must provide a compatible Core
Library.

The portable manifest records the requirement without an absolute local path.
The private ledger retains full evidence. Validation rejects any plan that
copies or rewrites a declared system dependency.

## Rejected Alternatives

- Treat every `RelativePathType = 5` reference as Core Library. This lacks
  enough evidence and can silently omit user-managed audio.
- Add type 5 rewrite support. Core Library references should remain managed by
  Ableton in the default mode.
- Hide the planner error only in the UI. That would leave the domain contract
  inconsistent.

## Consequences

- macOS Core Library dependencies no longer block current-path copies.
- Factory Packs and Windows Core Library classification remain unconfirmed and
  require separate fixtures.
- A future strict-portable mode may add an explicit collection policy without
  changing the default behavior.
