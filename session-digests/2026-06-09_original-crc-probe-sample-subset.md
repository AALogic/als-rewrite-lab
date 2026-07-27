# OriginalCrc Probe Sample Subset

Status: created  
Date: 2026-06-09  
Scope: copy a limited subset of real samples referenced by copied ALS files

## Purpose

Create a small local corpus for testing whether Ableton ALS `OriginalCrc` can be
reproduced from referenced audio files.

This is not a full Collect All and Save package.

## Source ALS Copies

Selected from:

```text
<private-corpus-root>/
```

Selected ALS ids:

```text
private_corpus_fixture_011.als
private_corpus_fixture_012.als
private_corpus_fixture_015.als
private_corpus_fixture_016.als
private_corpus_fixture_017.als
```

## Output Location

```text
<private-corpus-root>/crc_probe_sample_subset/
```

Main files:

```text
README.md
crc_probe_sample_subset_manifest.json
samples/
```

## Copy Result

```text
total copied samples: 21
total copied bytes: 255,328,326
folder size on disk: about 244 MB
```

Per ALS:

```text
11: 5 copied samples out of 9 active refs
12: 1 copied sample out of 1 active ref
15: 5 copied samples out of 120 active refs
16: 5 copied samples out of 27 active refs
17: 5 copied samples out of 112 active refs
```

## Safety

```text
Original ALS files were not modified.
Original audio files were not modified.
Only copied ALS files from the preserved corpus were analyzed.
Only a limited subset of existing referenced audio files was copied.
```

## Manifest Fields

For each copied sample, the manifest records:

```text
ALS ref_id
filename
relative_path_type
raw_path
raw_relative_path
ALS OriginalFileSize
ALS OriginalCrc
ALS DefaultDuration
ALS DefaultSampleRate
original source sample path
copied sample path
actual copied file size
SHA-256
CRC32 full file
CRC32 low16
Adler32 full file
Adler32 low16
quick comparison flags
```

## Initial Observation

Across the 21 copied samples:

```text
ALS OriginalFileSize matched actual copied file size for all copied samples.
ALS OriginalCrc did not match CRC32 low16 for these samples.
ALS OriginalCrc did not match Adler32 low16 for these samples.
```

Interpretation:

```text
This does not identify Ableton's OriginalCrc algorithm.
It weakens the simplest hypotheses: raw full-file CRC32 low16 and Adler32 low16.
More tests are needed against audio-data-only checksums, chunk-level checksums,
endianness variants, metadata-stripped audio, and controlled Ableton saves.
```

## Next Step

Use the manifest to test additional hypotheses:

```text
CRC32 over WAV/AIFF audio data only
CRC32 over selected audio chunks
CRC32 variants with different initialization/finalization
16-bit additive checksums
hash/checksum over decoded PCM data
controlled copy/rename/resave experiments in Ableton
```
