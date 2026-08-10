# Plan 028: TransferPayload v0.2

Status: accepted
Date: 2026-08-05

1. Replace the single copy-result candidate with one immutable collection
   identity, revision and ordered target list.
2. Register only a non-empty module-030 delivery snapshot.
3. Revalidate every target before arming an attempt.
4. Reject stale revisions, invalid members and concurrent attempts.
5. Preserve one-shot begin and explicit retry after terminal outcomes.
6. Keep paths private and provider behavior outside the module.
