# Explicit Candidate Selection, Live 11.3

Date: 2026-08-02
State: corrected package passed the second manual Ableton check

## Purpose

Prove the first source-bound explicit selection flow without exposing private
project names, filenames or local paths in Git.

## Fixture

The copied Live 11.3 project contained six active `AudioClip/SampleRef/FileRef`
references. Five audio files remained at their recorded paths. One audio file
was moved into a bounded candidate directory so the ALS-recorded path became
missing.

The explicit selection recorded:

```text
source ALS SHA-256
required asset ID
candidate native path
candidate SHA-256
selection schema version 0.1
```

Private selection, pipeline report and ledger files remain outside Git.

## Result

```text
run status: ready_for_manual_ableton_check
completed stage: promotion
current recorded-path bindings: 5
explicit user selections: 1
copy operations: 7
rewrite operations: 6
package validation: passed
promotion: promoted_ready_for_manual_check
```

Independent checks confirmed:

```text
all source ALS/audio hashes remained unchanged
all six copied audio hashes equal their selected sources
result ALS passes gzip and XML parsing
all six active references point to Samples/Imported
all six active RelativePathType values equal 3
```

## Remaining Gate

The first manual open reported all six media files missing. Investigation
confirmed that every copied file and rewritten reference existed, but the
generated package omitted the empty Ableton Project Info marker present in
Ableton project directories. The code now plans, creates, validates and records
that directory explicitly. This is a tested correction, but its causal effect
still requires a fresh generated package and another manual Live open.

The fresh v2 run completed with three planned, created and independently
verified directories. Its portable PackageManifest v0.2 records the
`Ableton Project Info` marker as verified. All six copied audio hashes match
the manifest, the source ALS hash is unchanged, and static validation and
promotion passed.

The corrected v2 ALS then opened successfully in Ableton Live 11.3 without
missing-media warnings. The earlier package, which omitted Ableton Project
Info, reported all six files missing. This paired result confirms the project
marker as a required generated-package invariant for this tested profile.

The remaining functional checks for this fixture are:

```text
no missing-media warning
all six clips play the expected audio
the project can be saved without repair prompts
```

Static success is not counted as runtime success until this gate is recorded.
