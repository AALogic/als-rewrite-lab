# Plan: 011 ALSRewriter

1. Validate snapshot-bound plan and staging contracts.
2. Locate exact direct FileRef values through XML structure and byte ranges.
3. Change only three approved values and verify the in-memory result.
4. Encode and validate a temporary ALS before replacing the staged copy.
5. Cover fail-closed behavior and semantic preservation with fixtures.

