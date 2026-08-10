# OriginalCrc Static Direction Dataset

Date: 2026-06-09
Status: completed

## Request

User asked to choose the best test data from the 20 copied ALS corpus for
static `OriginalCrc` research and copy referenced samples when needed.

Safety requirement:

```text
Copy files only.
Do not modify original ALS files.
Do not modify original sample files.
```

## Script

Created experiment helper:

```text
tools/experiments/python/original_crc_static_direction_dataset.py
```

The script:

```text
reads existing copied ALS files from the 20 ALS corpus
extracts active SampleRef/FileRef audio references
resolves referenced sample paths against the original ALS project locations
selects diagnostic sample cases for OriginalCrc research
copies selected ALS copies and selected audio files into an experiment folder
copies .asd sidecars when available
computes SHA-256 for copied files
checks source file stat after copy to confirm originals were not changed
```

## First Dataset

Created:

```text
<private-corpus-root>/original_crc_static_direction_dataset/
```

Result:

```text
copied ALS files: 7
copied sample files: 60
total copied sample bytes: 264448093
all source stat checks passed: true
```

This dataset is collision-heavy. It is useful, but less balanced.

## Recommended Balanced Dataset

Created:

```text
<private-corpus-root>/original_crc_static_direction_dataset_balanced/
```

Result:

```text
copied ALS files: 8
copied sample files: 60
total copied sample bytes: 144109271
all source stat checks passed: true
```

Category coverage:

```json
{
  "asd_sidecar_available": 12,
  "format_diversity": 12,
  "relative_path_type_diversity": 16,
  "small_pcm_probe_target": 16,
  "same_source_path_repeated": 5,
  "same_source_path_different_original_crc": 4,
  "same_original_crc_and_size": 24
}
```

Observed coverage:

```text
extensions: 33 .aif, 22 .wav, 5 .mp3
RelativePathType values: 0, 1, 3, 5
.asd sidecars copied/available for all 60 selected samples
```

## Why This Dataset Is Useful

It includes:

```text
Core Library samples with RelativePathType 5
project-local samples with RelativePathType 3
external/download/splice samples with RelativePathType 1
samples that also appear with RelativePathType 0 in the ALS corpus
same source path referenced many times
same source path with multiple OriginalCrc values
different files sharing OriginalCrc and OriginalFileSize
small files practical for decoded PCM tests
WAV, AIF and MP3 examples
.asd sidecar material for analysis-file correlation tests
```

## Current Interpretation

This dataset does not prove the `OriginalCrc` algorithm.

It is the preferred next static dataset for testing:

```text
same source path vs OriginalCrc stability
same OriginalCrc vs different file hashes
decoded PCM fingerprints
audio payload fingerprints
.asd correlation
RelativePathType / source-category correlation
```

## Safety Result

```text
Original ALS files: not modified
Existing ALS corpus copies: read and copied only
Original sample files: copied only
Source stat checks after copy: passed
```
