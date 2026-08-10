# Tasks 026: ExternalFolderHandoff

Status: v0.2 responsibility split accepted; live folder-drop verification pending
Date: 2026-08-05

## Readiness

- [x] DOCUMENTED_ONLY: handoff accepts only the latest successful module-018 result
- [x] DOCUMENTED_ONLY: provider URLs are backend-owned and closed in v0.2
- [x] DOCUMENTED_ONLY: drop does not claim remote upload success
- [x] DOCUMENTED_ONLY: live WeTransfer checks remain manual verification pending
- [x] REVIEW_ONLY: React owns no path validation, browser URL, upload or drag payload policy
- [x] REVIEW_ONLY: module 018 behavior and contracts remain unchanged
- [x] REVIEW_ONLY: native adapter never enumerates, copies, moves or deletes payload contents

## Contract Tests

- [x] ENFORCED_BY_TEST: unsupported_provider_is_rejected_before_effects
- [x] ENFORCED_BY_TEST: drag_operation_contract_is_copy_only
- [x] ENFORCED_BY_TEST: handoff_events_and_errors_are_path_free
- [x] ENFORCED_BY_TEST: handoff_wire_contract_is_v0_2_and_path_free
- [x] ENFORCED_BY_TEST: dom_surface_rect_is_converted_to_unflipped_appkit_coordinates
- [x] ENFORCED_BY_TEST: invalid_drag_surface_is_rejected
- [x] ENFORCED_BY_TEST: handoff_ipc_argument_case_is_explicit
- [x] ENFORCED_BY_TYPE: ExternalFolderHandoffRequest
- [x] ENFORCED_BY_TYPE: ExternalFolderHandoffPrepared
- [x] ENFORCED_BY_TYPE: ExternalFolderHandoffFinished
- [x] ENFORCED_BY_TYPE: ExternalFolderHandoffError

## Frontend Tests

- [x] FRONTEND_TEST: share_control_exists_only_for_successful_results
- [x] FRONTEND_TEST: share_armed_disables_window_drag_region
- [x] FRONTEND_TEST: share_drop_cancel_and_failure_restore_copy_outcome
- [x] FRONTEND_TEST: second_drag_requires_explicit_rearm
- [x] FRONTEND_TEST: share_status_copy_is_deterministic
- [x] FRONTEND_TEST: native_drag_started_event_must_match_the_armed_handoff

## Build

- [x] Consume the module-028 private payload attempt without accepting a path from React.
- [x] Add trusted-provider open, native-surface arm and cancel commands.
- [x] Add AppKit macOS directory drag adapter, first-click native surface and
  copy-only source.
- [x] Add fail-closed non-macOS adapter.
- [x] Add packaged parcel drag image.
- [x] Keep provider effects in QuickCopyAssistant and payload state outside the copy job.
- [x] Add pixel WeTransfer control and parcel sprite loop.
- [x] Run module guard, Rust, frontend and package verification.
- [x] Clear stuck sessions and native surfaces on new copy, new ALS and window close.

## Manual Verification

- [ ] OWNER_TEST: WeTransfer displays its own folder-drop hover treatment.
- [ ] OWNER_TEST: downloaded transfer preserves the folder tree without ALS Rescue ZIP creation.

These owner tests were deliberately not executed by Codex. No live provider
compatibility or remote upload-success claim is made before both are reported.

## Provider Opener v0.3

- [x] DOCUMENTED_ONLY: provider opening is independent from payload drag
- [x] REVIEW_ONLY: provider command receives no collection identity or path
- [x] ENFORCED_BY_TEST: trusted_provider_opens_without_arming_payload
- [x] ENFORCED_BY_TEST: provider_open_wire_contract_is_path_free
- [x] ENFORCED_BY_TYPE: ExternalProviderOpenRequest
- [x] ENFORCED_BY_TYPE: ExternalProviderOpenResult
- [x] ENFORCED_BY_TYPE: ExternalProviderOpenError

- [x] Replace combined handoff IPC with provider-only IPC.
- [x] Keep the trusted closed provider mapping.
