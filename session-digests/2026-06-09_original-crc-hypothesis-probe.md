# OriginalCrc Hypothesis Probe

Status: completed  
Date: 2026-06-09  
Scope: test checksum hypotheses against copied sample subset

## Input

```text
<private-corpus-root>/crc_probe_sample_subset/
```

Input subset:

```text
21 copied samples
5 copied ALS sources
WAV and AIFF/AIF files
about 244 MB on disk
```

## Probe 1: Python Fragment/Context Sweep

Output:

```text
<private-corpus-root>/crc_probe_sample_subset/hypothesis_probe/
```

Tested:

```text
fast CRC32 / Adler32 transforms over full file and contexts
CRC16 variants over small contexts/fragments up to 128 KB
CRC32 variants over small contexts/fragments
sum/checksum variants over small contexts/fragments
WAV data chunk / AIFF SSND payload contexts
first/last/middle fragments
```

Result:

```text
samples tested: 21
candidate values per sample: 392
total exact match rows: 0
samples with any exact match: 0
strong candidates: none
```

## Probe 2: Fast C Full/Payload Sweep

Output:

```text
<private-corpus-root>/crc_probe_sample_subset/fast_full_payload_probe/
```

Tested:

```text
CRC16 variants over full file
CRC16 variants over full WAV data / AIFF SSND payload
CRC32 variants low16/high16 over full file
CRC32 variants low16/high16 over full WAV data / AIFF SSND payload
simple sums
Fletcher16
byte-swapped variants
```

Result:

```text
samples tested: 21
match rows: 1
samples with any match: 1
candidate matching more than one sample: none
```

Only match:

```text
sample: private_audio_fixture_001.wav
context: audio_payload_wav_data
function: crc16_x25
value: 5963
```

## Interpretation

The common checksum hypotheses are not confirmed.

The result weakens these hypotheses:

```text
OriginalCrc = CRC16 variant over full file
OriginalCrc = CRC16 variant over WAV/AIFF audio payload
OriginalCrc = CRC32 low16/high16 over full file
OriginalCrc = CRC32 low16/high16 over audio payload
OriginalCrc = Adler32 low16/high16
OriginalCrc = simple byte/word sum
OriginalCrc = Fletcher16
```

One isolated match is not enough to infer an algorithm because `OriginalCrc`
appears to be 16-bit-like and many candidates were tested.

## Current Status

```text
OriginalFileSize is strongly useful: matched actual file size for all 21 samples.
OriginalCrc remains useful as Ableton-provided weak evidence.
OriginalCrc algorithm is still UNKNOWN.
Do not use OriginalCrc as sole identity proof.
```

## Next Hypotheses

More expensive/domain-specific tests remain:

```text
decoded PCM fingerprint
checksum after Ableton-style audio normalization
checksum over warp-analysis/cache-derived data
checksum over metadata from Ableton's internal sample object
controlled Ableton save experiments with renamed/copied/edited samples
```
