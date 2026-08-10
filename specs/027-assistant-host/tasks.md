# Tasks 027: AssistantHost

Status: accepted
Date: 2026-08-05

## Readiness

- [x] DOCUMENTED_ONLY: AssistantHost is a presentation host, not a plugin runtime
- [x] DOCUMENTED_ONLY: v0.1 hosts one existing courier experience
- [x] REVIEW_ONLY: AssistantHost owns no Tauri effects or capability policy
- [x] REVIEW_ONLY: module 018 and QuickCopy copy behavior remain unchanged
- [x] REVIEW_ONLY: licensing, provider catalogs and multiple characters stay outside scope

## Required Tests

- [x] FRONTEND_TEST: assistant_host_renders_supplied_status_without_domain_interpretation
- [x] FRONTEND_TEST: assistant_host_window_drag_region_follows_model
- [x] FRONTEND_TEST: assistant_host_keeps_payload_and_window_drag_geometry_separate
- [x] FRONTEND_TEST: assistant_host_exposes_supplied_controls_without_owning_actions

## Build

- [x] Add AssistantPresentationModel v0.1.
- [x] Extract AssistantHost component.
- [x] Add QuickCopy presenter composition.
- [x] Keep effect orchestration outside AssistantHost.
- [x] Run frontend tests, TypeScript check, build and module guard.

## Courier Presentation v0.2

- [x] DOCUMENTED_ONLY: presentation projects queue work and payload as separate facts
- [x] REVIEW_ONLY: AssistantHost owns no queue settings browser delivery or filesystem effect
- [x] REVIEW_ONLY: courier apparent scale remains stable outside working perspective
- [x] FRONTEND_TEST: queue_list_is_bounded_ordered_and_removes_only_removable_item
- [x] FRONTEND_TEST: arrival_collecting_working_and_ready_scenes_are_deterministic
- [x] FRONTEND_TEST: payload_hit_surface_is_separate_from_window_drag_surface
- [x] FRONTEND_TEST: context_menu_replaces_permanent_close_button
- [x] FRONTEND_TEST: reduced_motion_keeps_status_visible

- [x] Implement the v0.2 presentation model and scene.
- [ ] Add visual bounding-box and screenshot evidence.

## Reusable Courier Cycle Correction

- [x] Position the ready parcel in the courier's hands without scaling the courier.
- [x] Hide the held parcel while the native drag image follows the pointer.

## Integrated Held Payload v0.3

- [x] DOCUMENTED_ONLY: the held parcel is part of the character sprite strip
- [x] REVIEW_ONLY: the payload surface is invisible interaction geometry only
- [x] FRONTEND_TEST: ready_character_and_parcel_share_one_sprite
- [x] FRONTEND_TEST: native_drag_switches_to_empty_hands_without_duplicate_parcel
- [x] FRONTEND_TEST: cancelled_and_failed_drag_restore_integrated_parcel
- [x] FRONTEND_TEST: courier_canvas_scale_remains_stable_across_payload_states
- [x] Replace the loose parcel visual with the integrated held-payload strip.
- [x] Keep a transparent 48 by 48 native drag hit surface aligned to the parcel.
- [x] Keep complete icon-button rectangles interactive.
- [ ] Record refreshed visual bounding-box evidence in the packaged app.
