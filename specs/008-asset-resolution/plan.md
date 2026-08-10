# Plan 008: AssetResolution

Status: ready for policy v0.3 implementation
Date: 2026-08-02

1. Define proposal, candidate, evidence and decision contracts.
2. Validate upstream model versions and fatal states.
3. Generate a bounded candidate shortlist by exact path or filename signals.
4. Apply the versioned additive score and explicit conflicts.
5. Preserve an exact recorded-path candidate as a current binding without
   claiming historical identity.
6. Validate source-bound, path-and-SHA-256 user selections for moved candidates.
7. Apply ambiguity, stale-selection and partial-inventory gates.
8. Test every decision branch and a fake PackagePlanner consumer.
9. Run workspace quality commands and the module guard.
