# Fixture Contract: 017 CurrentPathCopyPipeline

Use synthetic copied Live 11.3 gzip/XML projects with direct supported
AudioClip FileRef records.

1. Complete fixture: two existing audio files. Expected complete copy, two
   relinks and no omissions.
2. Partial fixture: one existing and one missing audio file. Expected incomplete
   copy, one relink, one omission and unchanged missing FileRef values.
3. All-missing fixture: no existing audio. Expected structurally valid ALS-only
   incomplete copy with zero rewrites.
4. Safety fixtures: existing target, source mutation and filename collision
   remain blocking.
5. Project-local fixture: AudioClip and MultiSamplePart share one safe Type 3
   `Samples/Recorded/...` file. Expected one copied audio file, two Path-only
   rewrites, preserved RelativePath/RelativePathType and explicit directories.
6. Preview-drift fixture: prepare with one missing reference, create that file
   without changing ALS, then execute with the accepted fingerprint. Expected
   `preview_plan_changed` before staging, target or ledger creation.
7. Matching fingerprint fixture: unchanged source and audio state executes the
   exact reviewed plan successfully.

Hash source ALS and source audio before and after every write-capable test.
