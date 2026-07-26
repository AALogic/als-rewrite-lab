# ADR-003: References, Requirements, Locations And Content Are Separate

Status: accepted  
Date: 2026-07-26

## Context

One audio file may appear in many ALS reference locations. The same bytes may
also exist at several filesystem paths, while two different files may have the
same name, size or Ableton `OriginalCrc`.

A catch-all dependency record cannot safely represent all these meanings.

## Decision

The domain separates:

```text
ReferenceOccurrence
  one observed reference occurrence in ALS

RequiredAsset
  one logical audio requirement grouping one or more references

FileOccurrence
  one file observed at one native location, volume and time

ContentIdentity
  content identity supported by strong evidence
```

`DependencyRef v0.1` remains a compatibility representation of
`ReferenceOccurrence` until a versioned downstream contract replaces it.

## Consequences

- DependencyExtractor does not deduplicate occurrences.
- Grouping occurrences into RequiredAsset belongs to DependencyAssessment.
- PathObservation does not create ContentIdentity.
- AssetResolution owns evidence and decisions connecting requirements to files.
- Counts in reports must say whether they count references, requirements,
  locations or unique content.
- Migrations must be additive and versioned; existing 001/002 code is not
  rewritten only for naming consistency.
