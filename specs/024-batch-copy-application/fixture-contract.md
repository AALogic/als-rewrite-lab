# Fixture Contract 024: BatchCopyApplicationService

Status: verified

Tests use deterministic fake one-project operations for orchestration behavior.
The fake records every prepare and execute call, returns configured successful
or failed Desktop previews/results, and never accesses user paths.

Required scenarios:

```text
two ready selections in explicit order
one blocked preview followed by one ready preview
two distinct selections with the same proposed target
first execution failure followed by success
cancellation requested after the first completed job
missing write consent
aggregate diagnostic redaction canary paths
shared JSON wire fixture accepted by Rust and TypeScript
```

Existing module 017 and Desktop copy tests remain the evidence for actual
copy/rewrite/validation/manifest behavior. Module 024 tests must not duplicate
that policy.
