# Plan: 010 StagingExecutor

1. Define a small execution contract separate from PackagePlanner.
2. Validate the complete request before the first write.
3. Copy through create-new temporary files and verify full hashes.
4. Preserve a structured operation record for later PrivateLedger.
5. Prove fail-closed and non-destructive behavior with filesystem tests.
