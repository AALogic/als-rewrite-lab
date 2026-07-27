# Plan: 012 Validator / SemanticDiff

1. Validate run identities and completion states.
2. Independently hash and enumerate staged package files.
3. Independently compare source and rewritten ALS XML.
4. Mask only explicitly approved field ranges and reject all other differences.
5. Return a pure evidence result used as the promotion gate.

