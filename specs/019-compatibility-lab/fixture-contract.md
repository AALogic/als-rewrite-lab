# Fixture Contract: 019 Compatibility Lab

Use synthetic gzip/XML fixtures that copy the confirmed direct reference shapes
but vary document metadata. No private ALS or audio enters Git.

1. Strict baseline: Live 11.3 label, direct AudioClip Type 1, existing WAV.
2. Unconfirmed known shape: Live 10 or Live 11.2 label with the same supported
   direct Type 1 structure. Strict mode blocks; lab mode may copy after consent.
3. Known Type 3: unconfirmed version with safe `Samples/Recorded/...`; lab mode
   uses Path-only relocation.
4. Unknown shape: unsupported parent context, unknown RelativePathType or unsafe
   relative traversal. Both modes block before staging.
5. Privacy fixture: source, project and sample names contain unique sentinel
   strings. No sentinel may appear in copied reports.
6. Manual verification fixture: each allowed manual outcome maps to its stated
   provisional conclusion without changing the underlying machine reports.

Hash source ALS and audio before and after every write-capable fixture.
