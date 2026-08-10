# Plan 020: ALSProjectScanner

Status: complete
Date: 2026-08-03

1. Add versioned Project scan contracts and a progress observer boundary to
   `rescue_catalog`.
2. Validate the scan ID, policy, positive entry budget, absolute roots and
   absolute exclusions without canonicalizing paths.
3. Reduce duplicate and lexically overlapping roots deterministically.
4. Implement iterative, sorted traversal with lexical exclusions, optional
   depth limit, cancellation checks and no symlink following.
5. Observe only regular `.als` files and exact real `Ableton Project Info`
   directories; never open file content.
6. Normalize observations and identifiers after sorting.
7. Test request failures, partial coverage, cancellation, privacy-safe
   progress, determinism, read-only behavior and downstream handoff.
8. Run module-ready before implementation and the full Rust/module verification
   sequence before acceptance.

The module does not refactor `AssetInventory` in this change. Both scanners
have different contracts and policy. A shared walker may be considered only
after both behaviors are stable and tests prove a refactor preserves them.
