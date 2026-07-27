# ADR-005: Snapshot-Bound Rewrite Evidence Handoff

Status: accepted for laboratory rewrite
Date: 2026-07-27

## Context

DependencyExtractor intentionally emits occurrence and dependency facts without
rewrite locators. Package planning and ALS rewrite nevertheless require an
exact active XML target. Adding locator fields to every downstream dependency
contract would couple read-only analysis to writer internals.

## Decision

ALSReadModel has a second, explicit consumer handoff for rewrite planning:

~~~text
SetMetadata:
  source_als_path
  source_file_hash
  ableton_document_version
  ableton_creator_version
  ableton_minor_version

ActiveAudioReference:
  ref_id
  raw_path
  raw_relative_path
  relative_path_type
  xml_locator
  is_rewrite_candidate
  rewrite_support_status
~~~

PackagePlanner joins this snapshot-bound evidence to assessment occurrences by
`als_ref_id`. The handoff does not flow through DependencyRef and does not
change DependencyExtractor v0.1.

Every rewrite operation records the source ALS hash, expected old values,
locator and ruleset. A source hash change, locator mismatch, unexpected old
value or unsupported document/rule blocks execution.

## Consequences

- read-only analysis contracts remain small;
- writer coupling is explicit and isolated;
- locator index is never treated as cross-save identity;
- E-03 support matrix controls which references can become rewrite operations;
- a PackagePlan may contain copy-only work while rewrite remains blocked.
