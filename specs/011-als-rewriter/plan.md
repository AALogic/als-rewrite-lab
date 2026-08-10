# Plan: 011 ALSRewriter

1. Validate snapshot-bound plan and staging contracts.
2. Locate exact direct FileRef values through XML structure and byte ranges.
3. Change only the rule-specific approved values and verify both changed and
   intentionally unchanged path fields in memory.
4. Encode and validate a temporary ALS before replacing the staged copy.
5. Cover fail-closed behavior and semantic preservation with fixtures.
6. Keep platform replacement behind the I/O adapter and prove native Windows
   success, locked-file preservation, Unicode, and cleanup behavior.
