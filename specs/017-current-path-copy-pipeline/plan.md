# Plan: 017 CurrentPathCopyPipeline

1. Add exact-file snapshotting to AssetInventory.
2. Add non-blocking omissions only for genuinely missing current-path assets.
3. Preserve safe Type 3 Samples/... placement and rewrite only absolute Path.
4. Compose read, snapshot, exact-path resolution, plan, write and promotion.
5. Allow validation, manifests and promotion for complete and incomplete plans.
6. Add complete, partial, Type 3 and safety integration fixtures.
7. Return the canonical package-plan fingerprint from preview and execution.
8. Compare an expected fingerprint before staging and fail closed on drift.
