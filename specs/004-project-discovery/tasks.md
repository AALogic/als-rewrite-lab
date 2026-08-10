# Tasks 004: ProjectDiscovery

Status: ready
Date: 2026-07-27

## Readiness

- [x] DOCUMENTED_ONLY: E-01 confirms exact Ableton Project Info marker evidence
- [x] DOCUMENTED_ONLY: parent(ALS) alone is not Project root evidence
- [x] REVIEW_ONLY: module remains bounded ancestor metadata inspection
- [x] REVIEW_ONLY: no main-Set decision is introduced

## Contract

- [x] ENFORCED_BY_TEST: exact_marker_confirms_project_root
- [x] ENFORCED_BY_TEST: nested_set_uses_marker_bearing_ancestor
- [x] ENFORCED_BY_TEST: backup_set_is_labeled_without_main_set_claim
- [x] ENFORCED_BY_TEST: absent_marker_keeps_project_root_unknown
- [x] ENFORCED_BY_TEST: case_variant_marker_is_not_confirmed
- [x] ENFORCED_BY_TEST: nested_markers_are_ambiguous
- [x] ENFORCED_BY_TEST: marker_symlink_is_not_confirmed
- [x] ENFORCED_BY_TEST: source_symlink_is_rejected
- [x] ENFORCED_BY_TEST: discovery_is_read_only
- [x] ENFORCED_BY_TYPE: ProjectDiscoveryRequest
- [x] ENFORCED_BY_TYPE: ProjectDiscoveryResult
- [x] ENFORCED_BY_TYPE: ProjectDiscoveryMetadata
- [x] ENFORCED_BY_TYPE: ProjectRootCandidate
- [x] ENFORCED_BY_TYPE: ProjectDiscoveryWarning
- [x] ENFORCED_BY_TYPE: ProjectDiscoveryError

## Build

- [x] Create rescue_analyzer crate.
- [x] Implement project_discovery.rs.
- [x] Implement project_discovery_impl.rs.
- [x] Run all workspace quality commands.
- [x] Run workflow_guard verify-module 004-project-discovery.
