# ADR-010: Separate Compatibility Lab Profile

Status: accepted
Date: 2026-08-03

## Context

The strict Desktop Alpha authorizes rewrite only for the confirmed Live 11.3
document profile. Private Windows test hosts contain useful Live 10 and Live
11.2 projects. Rejecting those documents before structural inspection prevents
the project from gathering the evidence needed to confirm or reject support.

Removing the version gate globally would weaken the normal product and turn an
unconfirmed ALS version into an automatic rewrite rule.

## Decision

Build a separate `ALS Rescue Compatibility Lab` application from the same
repository.

The normal application remains strict. The Compatibility Lab build may request
`compatibility_lab_current_paths_copy`, but only after:

1. the user gives explicit experimental consent;
2. every reference selected for rewrite matches a known structural profile;
3. the existing plan, staging, rewrite, semantic-diff, validation, manifest and
   promotion gates pass;
4. the result remains `ready_for_manual_check`, never confirmed compatible.

Version metadata is evidence and reporting context. It is not sufficient
rewrite authorization. Unknown reference structures still fail closed.

The lab build has a distinct product name and application identifier so it can
coexist with the strict build. A compile-time feature prevents the experimental
desktop request from being accepted by an ordinary build.

## Reporting

The application produces a redacted, copyable compatibility report containing
build/host facts, ALS version metadata, structural reference profiles, planning
and execution outcomes, safe errors and the user's manual Ableton result. It
must not contain local paths, project names, sample names or raw XML values.

Manual success creates only `candidate_success`. Promotion to a confirmed
support profile requires reviewed evidence and a later explicit product
decision.

## Consequences

- One codebase owns strict and research behavior without duplicating modules.
- Existing safety boundaries remain active in both builds.
- Real Live 10 and 11.2 tests can produce useful evidence.
- The compatibility report schema becomes a versioned application contract.
- The lab installer is a private unsigned research artifact, not a commercial
  compatibility claim.
