# Session Digest: ALS Structure Corpus 20

Status: preserved digest  
Date: 2026-06-02  
Mode: Research / Product Office memory

## 1. User Intent

User wanted a broader ALS exploration before continuing product/spec work.

Goal:

```text
Find 20 new ALS files, copy them into a safe experiment folder, analyze the
copies deeply and preserve what we learn so context does not disappear.
```

## 2. What Was Done

Created experiment:

```text
experiments/2026-06-02_als_structure_corpus_20/
```

Created research helper:

```text
tools/experiments/python/als_structure_corpus_probe.py
```

The helper:

```text
found ALS candidates
selected 20 copied ALS files
copied only .als files into the experiment folder
decompressed gzip XML from copies
counted tags, FileRef contexts, OriginalFileRef contexts, active refs,
path classes, RelativePathType values and interesting ALS structures
generated JSON/CSV/Markdown reports
```

Original Ableton projects were not modified.

## 3. Where Results Live

Start here:

```text
experiments/2026-06-02_als_structure_corpus_20/README.md
```

Human interpretation:

```text
experiments/2026-06-02_als_structure_corpus_20/findings.md
```

Generated report:

```text
experiments/2026-06-02_als_structure_corpus_20/structure_report.md
```

Machine-readable summary:

```text
experiments/2026-06-02_als_structure_corpus_20/reports/corpus_summary.json
```

Per-file reports:

```text
experiments/2026-06-02_als_structure_corpus_20/reports/per_file/
```

## 4. Key Evidence

Corpus totals:

```text
files: 20
compressed ALS bytes: 53,019,906
decompressed XML bytes: 765,908,275
XML element count: 15,646,392
unique XML tag names: 17,696
SampleRef count: 114,253
all FileRef count: 121,268
active SampleRef/FileRef count: 114,253
non-sample FileRef count: 7,015
OriginalFileRef count: 4,084
SampleRef without direct FileRef: 0
```

RelativePathType counts for active refs:

```text
3 -> 64,362
0 -> 46,842
1 -> 2,316
5 -> 733
```

Active path classes:

```text
absolute_user_path: 54,105
project_samples_like: 46,842
absolute_downloads: 12,573
absolute_other: 733
```

## 5. Findings

FACT:

```text
Direct SampleRef/FileRef is still a good observed active audio dependency rule
in this corpus.
```

FACT:

```text
RelativePathType 0 exists in active refs and appears often with Path like
Samples/Processed/... and empty RelativePath.
```

FACT:

```text
FileRef exists outside active sample dependency contexts.
Global FileRef rewrite would be unsafe.
```

FACT:

```text
OriginalFileRef appears in sample provenance and also device/source contexts.
It must remain separate from active refs.
```

FACT:

```text
Active sample refs appear in multiple contexts: audio clips, take lanes,
session clip slots, arrangement events and OriginalSimpler/MultiSamplePart.
```

HYPOTHESIS:

```text
RelativePathType 0 may represent project-relative path stored directly in Path.
```

HYPOTHESIS:

```text
ALSReader should preserve xml_context / usage_context for downstream modules.
```

## 6. Product Consequence

This experiment strengthens the case that ALSReader needs two output layers:

```text
raw/internal dependency model
  used by ProjectAnalyzer, PackagePlanner, ALSRewriter, Validator

user-facing diagnostic JSON
  used by CLI/UI reports
```

The current ALSReader JSON is useful as a diagnostic slice, but probably too
narrow as the only downstream contract.

## 7. Next Step

Use this digest and experiment folder during:

```text
ALSReader Contract Review
```

Main review question:

```text
Should ALSReader v0.2 add a raw/internal dependency model that preserves
xml_context, RelativePathType 0 and raw FileRef fields for downstream modules,
while keeping CLI JSON as a simpler diagnostic report?
```

## 8. Important Limitation

This is research evidence, not final rewrite policy.

Do not infer official Ableton semantics from this corpus alone.

Do not implement rewrite rules from this without a focused spec, fixtures and
validation.
