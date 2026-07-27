# Fixture Contract 003: PathObservation

Status: ready; E-01 accepted
Date: 2026-07-27
Scope: synthetic evidence for read-only candidate path observations

## 1. Purpose

PathObservation records candidate path evidence without selecting a candidate
or claiming asset identity.

Required boundary:

```text
PathObservation checks explicit candidate paths only.
PathObservation does not search the whole disk.
PathObservation does not classify source category.
PathObservation does not deduplicate reference occurrences.
PathObservation does not select a candidate.
PathObservation does not rewrite ALS.
```

## 2. Fixture Policy

Committed fixtures for this module must be synthetic and must not contain:

```text
private Ableton projects
usernames or absolute user home paths
licensed audio
real client or project names
```

Use temporary directories and files created by the Rust tests. Tests may create
zero-filled or short synthetic files because this module reads metadata only.

## 3. Required Fixture Matrix

### 3.1 Confirmed Project Root

Input:

```text
project_root_basis: controlled_fixture
confirmed_project_root: native temporary fixture root
raw_relative_path: Samples/Imported/Kick.wav
```

Expected:

```text
one candidate_basis: confirmed_project_root_plus_raw_relative_path
candidate remains an observation
identity_status: not_evaluated
```

### 3.2 No Confirmed Project Root

Expected:

```text
raw_relative_path is preserved
no project-relative candidate is created
warning explains missing confirmed context
```

### 3.2A RelativePathType 0

Input:

```text
relative_path_type: 0
raw_path: Samples/Processed/Consolidate/Kick.aif
raw_relative_path: empty
```

Expected:

```text
candidate_basis: confirmed_project_root_plus_raw_path
candidate is contained lexically under confirmed_project_root
```

### 3.2B RelativePathType 3

Input:

```text
relative_path_type: 3
raw_path: native absolute path
raw_relative_path: Samples/Imported/Kick.wav
```

Expected:

```text
one confirmed_project_root_plus_raw_relative_path candidate
one recorded_raw_absolute_path candidate when native
neither candidate is selected
```

### 3.2C RelativePathType 1 And 5

Expected:

```text
relative fields are preserved but are not joined to confirmed_project_root
a native absolute raw_path may still produce recorded_raw_absolute_path
```

### 3.3 Two Existing Candidates

Input contains a safe project-relative candidate and a checkable absolute raw
path. Both point to regular files.

Expected:

```text
two CandidatePathObservation records
no selected_path_candidate
no verified status
deterministic order without preference meaning
```

### 3.4 Missing Candidate

Expected:

```text
availability_status: missing
entry_kind: unknown
identity_status: not_evaluated
```

### 3.5 Existing Directory

Expected:

```text
availability_status: existing_directory
entry_kind: directory
```

### 3.6 Existing Symlink

Expected:

```text
availability_status: existing_symlink
entry_kind: symlink
symlink target is not followed
warning includes PATH_CANDIDATE_IS_SYMLINK
```

### 3.7 Parent Escape

Input:

```text
raw_relative_path: ../../outside.wav
```

Expected:

```text
safety_status: rejected_parent_escape
availability_status: not_checked
no metadata read outside confirmed_project_root
```

### 3.8 Size Evidence

For matching and differing fixture sizes:

```text
size_evidence_status records match or difference
identity_status remains not_evaluated
no candidate is selected
```

### 3.9 Foreign Platform Path

On macOS use Windows drive and UNC strings; on Windows use a macOS absolute
string.

Expected:

```text
raw value preserved exactly
platform_status: foreign_platform_path
availability_status: not_checked
```

### 3.10 Duplicate Reference Occurrences

Expected:

```text
two DependencyRef inputs produce two DependencyPathObservation outputs
dependency_id and als_ref_id remain attached
no grouping or deduplication
```

### 3.11 Unicode And Case

Fixtures must include:

```text
spaces
NFC and NFD names
case variants
the longest path practical for unit tests
```

Expected:

```text
raw values are preserved exactly
no Unicode normalization is used as an identity rule
case variants remain distinct path observations
native filesystem metadata determines only observed availability
```

### 3.12 Metadata-Only Safety

Snapshot the temporary directory before and after observation.

Expected:

```text
no created, deleted, renamed or modified file
no followed symlink
no opened audio content
```

## 4. Experiment Gate

Accepted evidence:

```text
E-01/E-02 report:
  docs/experiments/E-01-E-02-path-and-coverage-2026-07-27.md

private evidence:
  read-only corpus
  SHA-256 manifest retained outside Git

accepted project-relative types:
  0 -> raw_path
  3 -> raw_relative_path
```
