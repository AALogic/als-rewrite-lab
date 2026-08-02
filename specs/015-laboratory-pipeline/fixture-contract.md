# Fixture Contract: 015 LaboratoryPipeline

Automated tests create a complete synthetic ALS, audio source, scan scope,
exact Ableton Project marker, staging root, target root, and private-ledger
parent under a temporary directory.
All destructive setup operations affect only that temporary fixture.

Safety variants cover an absent marker, direct and ancestor-aliased outputs
inside the confirmed Project, case-variant aliases on case-insensitive hosts, a
zero-reference plan, and a dangling output symlink where the platform supports
symlink fixtures.

The real-data laboratory run uses only ALS and audio copies under the dedicated
overnight laboratory directory. Original user projects and samples remain
read-only and their hashes are checked before and after the run. Private
evidence may contain local paths and must remain outside the portable package.

The successful policy v0.3 fixture uses a Live 11.3 `AudioClip`, one moved
audio candidate and a `UserSelectionSet v0.1` bound to the source ALS SHA-256,
candidate native path and candidate SHA-256. Changed selected content and a
selection for another ALS snapshot block before staging.
