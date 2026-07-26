# ALSReader v0.2 Corpus Smoke 3

Status: PASS  
Date: 2026-06-09  
Scope: run current `ALSReader v0.2` on 3 copied ALS files from the preserved corpus

## Purpose

Check whether the implemented module behaves correctly on copied real-world ALS
projects, not only on small fixture tests.

Expected behavior:

```text
parse copied ALS files
return ALSReadModel v0.2
preserve active vs historical references separately
preserve raw RelativePathType values, including 0
do not output exists_on_disk
do not emit active-path-missing warnings
do not modify copied ALS files
```

## Tested Copies

```text
experiments/2026-06-02_als_structure_corpus_20/copies/11__FILIP-SB-REMIX.als
experiments/2026-06-02_als_structure_corpus_20/copies/14__POLISHBOYS_WWA_11.10_GOSCINKA.als
experiments/2026-06-02_als_structure_corpus_20/copies/20__acidsynth.als
```

## Results

### 11__FILIP-SB-REMIX.als

```text
read_only_hash_unchanged: true
model_version: 0.2
reader_version: 0.2.0
sample_ref_count: 9
active_audio_ref_count: 9
historical_ref_count: 22
non_audio_signal_count: 45
warning_count: 0
error_count: 0
contains_exists_on_disk: false
contains_active_path_missing_warning: false
relative_path_type_counts: 1 -> 9
```

### 14__POLISHBOYS_WWA_11.10_GOSCINKA.als

```text
read_only_hash_unchanged: true
model_version: 0.2
reader_version: 0.2.0
sample_ref_count: 1800
active_audio_ref_count: 1800
historical_ref_count: 48
non_audio_signal_count: 111
warning_count: 0
error_count: 0
contains_exists_on_disk: false
contains_active_path_missing_warning: false
relative_path_type_counts: 0 -> 1100, 3 -> 700
```

First observed active `RelativePathType 0` example:

```text
raw_path: Samples/Processed/Consolidate/MKS KICK [2023-05-20 174929]-1-1-1.aif
raw_relative_path: ""
xml_context: SampleRef/FileRef
rewrite_support_status: requires_test
```

### 20__acidsynth.als

```text
read_only_hash_unchanged: true
model_version: 0.2
reader_version: 0.2.0
sample_ref_count: 456
active_audio_ref_count: 456
historical_ref_count: 237
non_audio_signal_count: 123
warning_count: 0
error_count: 0
contains_exists_on_disk: false
contains_active_path_missing_warning: false
relative_path_type_counts: 1 -> 216, 5 -> 240
```

## Interpretation

ALSReader v0.2 behaved as intended for this smoke set.

Confirmed:

```text
The module parses copied real-world ALS files.
The module keeps source ALS copies byte-identical.
The module returns ALSReadModel v0.2.
The module does not check sample path existence.
The module does not output exists_on_disk.
The module preserves RelativePathType 0 without treating it as an error.
The module separates active audio refs, historical refs and report-only signals.
```

Still not proven:

```text
xml_locator stability for rewrite
semantic correctness of usage_context
full handling of every file in the 20-file corpus
PathVerifier behavior
rewrite behavior
```
