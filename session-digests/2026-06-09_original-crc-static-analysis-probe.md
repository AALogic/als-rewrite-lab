# OriginalCrc Static Analysis Probe

Date: 2026-06-09
Status: completed

## Request

Run the static tests discussed for Ableton ALS `OriginalCrc`:

```text
same SHA-256 vs OriginalCrc
OriginalCrc collisions
decoded PCM
.asd
audio metadata correlation
```

## Dataset

Recommended balanced dataset:

```text
experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset_balanced/
```

Scope:

```text
copied samples: 60
copied ALS files: 8
copied .asd sidecars: 60
formats: .aif, .wav, .mp3
RelativePathType values: 0, 1, 3, 5
```

Safety:

```text
Only copied experiment files were read.
Original Ableton projects were not modified.
Original sample files were not modified.
```

## Script

Created and run:

```text
tools/experiments/python/original_crc_static_analysis_probe.py
```

Local tooling used:

```text
/usr/bin/afinfo
/usr/bin/afconvert
tools/experiments/c/original_crc_fast_probe.c
```

## Output

Results saved in:

```text
experiments/2026-06-02_als_structure_corpus_20/original_crc_static_direction_dataset_balanced/static_analysis_probe/
```

Key files:

```text
static_analysis_report.json
samples_enriched.csv
same_sha256_vs_original_crc.csv
original_crc_collisions.csv
same_pcm_sha256_vs_original_crc.csv
decoded_pcm_checksum_matches.csv
same_asd_sha256_vs_original_crc.csv
asd_checksum_matches.csv
asd_embedded_original_crc_matches.csv
metadata_candidate_matches.csv
metadata_signature_groups.csv
```

## Results

### 1. Same SHA-256 vs OriginalCrc

```text
sample count: 60
unique SHA-256 count: 52
SHA groups with more than one sample: 8
SHA groups with multiple nonzero OriginalCrc values: 0
```

Interpretation:

```text
When the copied file bytes are identical, nonzero OriginalCrc stayed stable in
this dataset.
```

This supports using `OriginalCrc` as weak supporting evidence, but does not
prove the algorithm.

### 2. OriginalCrc Collisions

```text
unique nonzero OriginalCrc values: 37
OriginalCrc values used by more than one sample: 9
OriginalCrc values used by more than one SHA-256: 6
largest collision group: 5 samples
```

Examples:

```text
OriginalCrc 44604 -> 5 samples, 4 distinct SHA-256, 4 distinct PCM hashes
OriginalCrc 54113 -> 5 samples, 5 distinct SHA-256, 5 distinct PCM hashes
OriginalCrc 56098 -> 5 samples, 3 distinct SHA-256, 3 distinct PCM hashes
OriginalCrc 63309 -> 5 samples, 3 distinct SHA-256, 3 distinct PCM hashes
```

Interpretation:

```text
OriginalCrc is not unique enough to identify a sample by itself.
```

### 3. Decoded PCM

All 60 files decoded successfully through `afconvert`.

```text
decoded sample count: 60
unique decoded PCM data hashes: 52
PCM hash groups with more than one sample: 8
PCM hash groups with multiple nonzero OriginalCrc values: 0
checksum match rows: 1
```

The only checksum match was:

```text
sample 40
candidate: decoded_pcm_wav:audio_payload_wav_data:crc32_jamcrc:high16_byteswap
expected OriginalCrc: 18464
```

Interpretation:

```text
One checksum match is weak evidence and can happen by chance.
Decoded PCM identity is stable with OriginalCrc in duplicate cases, but
OriginalCrc has collisions across different PCM hashes.
```

So `OriginalCrc` is not a strong unique PCM fingerprint.

### 4. ASD Sidecars

```text
samples with .asd: 60
unique .asd SHA-256 count: 52
.asd hash groups with more than one sample: 8
.asd hash groups with multiple nonzero OriginalCrc values: 0
.asd checksum match rows: 0
```

Literal `OriginalCrc` byte-search inside `.asd`:

```text
embedded OriginalCrc rows: 14
uint16_le rows: 9
uint16_be rows: 5
```

Interpretation:

```text
No direct .asd checksum hypothesis matched.
The byte-search matches are weak unless future tests show a stable offset or
stable structure.
```

### 5. Audio Metadata Correlation

```text
metadata candidate match rows: 0
format signatures with multiple OriginalCrc values: 7
duration signatures with multiple OriginalCrc values: 1
ALS audio signatures with multiple OriginalCrc values: 1
```

No tested simple metadata value matched `OriginalCrc`:

```text
file size
audio bytes
audio packets
sample rate
default sample rate
default duration
estimated frame count
duration in ms
low16 / high16 / byteswapped variants
```

Interpretation:

```text
OriginalCrc does not look like a simple formula over obvious audio metadata.
```

## Current Conclusion

The strongest current interpretation:

```text
OriginalCrc is stable for identical copied file/PCM/.asd cases.
OriginalCrc is not unique and collides across different files and different PCM.
OriginalCrc is not explained by common checksum functions tested so far.
OriginalCrc is not explained by obvious metadata fields.
OriginalCrc may be a short Ableton-side fingerprint/checksum, but it remains
UNKNOWN.
```

## Product Rule

Keep current product rule:

```text
Do not use OriginalCrc as sole proof of identity.
Use OriginalCrc only as weak supporting evidence.
Prefer strong file hash, file size, decoded PCM/audio fingerprint, path/name
evidence and user confirmation for ambiguous matches.
```

## Recommended Next Step

Static analysis has narrowed the space. The next high-value step is a controlled
Ableton save experiment:

```text
same bytes different path
same bytes different filename
same PCM changed metadata
same size changed audio
Collect All and Save
Save As
.asd regeneration
```

This is more likely to reveal what Ableton recomputes and when.
