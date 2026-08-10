# Tasks 025: QuickCopyAssistant

Status: accepted
Date: 2026-08-05

## Readiness

- [x] DOCUMENTED_ONLY: quick mode reuses unchanged module 018 copy policy
- [x] DOCUMENTED_ONLY: one Play is the explicit write consent for complete or incomplete execution
- [x] DOCUMENTED_ONLY: v0.1 handles one ALS and does not watch Finder selections
- [x] REVIEW_ONLY: React owns no ALS, package, rewrite or incomplete-copy policy
- [x] REVIEW_ONLY: internal structured failures collapse only at the quick presentation boundary
- [x] REVIEW_ONLY: character animation never hides or changes operation status

## Contract Tests To Implement

- [x] ENFORCED_BY_TEST: valid_single_als_open_request_is_accepted
- [x] ENFORCED_BY_TEST: non_als_open_request_is_refused
- [x] ENFORCED_BY_TEST: multiple_opened_urls_are_refused_in_quick_v0_1
- [x] ENFORCED_BY_TEST: quick_launch_context_wire_contract_is_stable
- [x] REVIEW_ONLY: quick mode reuses DesktopCopyPreview and DesktopCopyResult without policy fields
- [x] FRONTEND_TEST: unable_preview_never_executes_copy
- [x] FRONTEND_TEST: one_play_executes_incomplete_preview_without_second_prompt
- [x] FRONTEND_TEST: quick_state_controls_follow_contract
- [x] FRONTEND_TEST: quick_status_copy_is_deterministic
- [x] ENFORCED_BY_TYPE: QuickCopyLaunchContext

## Build

- [x] Register `.als` as `Viewer` / `Alternate` in bundle configuration.
- [x] Add and initialize the single-instance plugin first.
- [x] Add macOS opened-URL launch adapter and one-ALS validation.
- [x] Add quick-window creation, reuse and cursor-relative placement.
- [x] Route the React root by Tauri window label.
- [x] Add the pure copy-job reducer and presentation controls.
- [x] Keep transfer-payload state outside the copy-job contract.
- [x] Reuse Folder, target suggestion, prepare and execute commands.
- [x] Create the original base character and state sprite strips.
- [x] Add CSS step animations and reduced-motion fallback.
- [x] Add Rust, TypeScript and wire-contract tests.
- [x] Run module-ready before coding and verify-module before acceptance.
- [x] Build and run the packaged macOS fixture matrix.

## Manual Acceptance

- [x] Ableton remains the default `.als` opener.
- [x] Cold Open With shows no main-window flash.
- [x] Warm Open With reuses the same application process.
- [x] The panel appears near the cursor and clamps to the active display work area.
- [x] A complete real copied project opens in Ableton; incomplete output is structurally valid and reports its omission.
- [x] Source ALS and source audio remain unchanged.

## Courier Collection v0.2

- [x] DOCUMENTED_ONLY: every accepted launch becomes one ordered queue item
- [x] DOCUMENTED_ONLY: Play starts immutable module-024 waves through module 030
- [x] REVIEW_ONLY: QuickCopy owns no queue merge or delivery policy
- [x] ENFORCED_BY_TEST: multiple_opened_urls_are_routed_in_order
- [x] ENFORCED_BY_TEST: warm_launch_appends_to_active_collection
- [x] ENFORCED_BY_TEST: finder_drop_reuses_quick_als_validation
- [x] FRONTEND_TEST: new_intake_after_ready_returns_to_collecting
- [x] FRONTEND_TEST: intake_during_processing_stays_visible_and_ordered

- [x] Replace the one-project React flow with collection commands and events.
- [x] Preserve cold/warm Open With behavior and run packaged smoke tests.

## Reusable Courier Cycle Correction

- [x] FRONTEND_TEST: new_intake_overrides_a_stale_delivered_presentation_state
- [x] FRONTEND_TEST: native_drag_switches_to_empty_hands_without_duplicate_parcel
- [x] FRONTEND_TEST: ready_delivery_controls_do_not_repeat_the_destination_folder_action
- [x] Reset the payload presentation when queued or processing work becomes active.
- [x] Keep the destination action out of the completed delivery step.
- [x] Increase the visible arrival duration and stabilize complete control hitboxes.
- [x] Reapply floating and full-screen-space window policy when the quick window is shown.

## Terminal Delivery v0.3

- [x] DOCUMENTED_ONLY: a successful handoff ends the active order
- [x] REVIEW_ONLY: completion copy is derived from backend handoff facts
- [x] FRONTEND_TEST: completed_handoff_messages_match_the_delivery_channel
- [x] FRONTEND_TEST: completed_handoff_hides_parcel_and_delivery_controls
- [x] FRONTEND_TEST: retryable_handoff_keeps_the_parcel_and_delivery_controls
- [x] FRONTEND_TEST: native_handoff_keeps_attempt_until_terminal_event
- [x] FRONTEND_TEST: assistant_host_names_completed_retry_as_send_again
- [x] Display backend-owned terminal outcomes and remove the delivered parcel.
- [x] Reopen an immutable package only through explicit Send Again.
- [x] Verify fresh item-one presentation after terminal delivery.

## macOS Overlay Presence v0.4

- [x] DOCUMENTED_ONLY: quick mode uses an isolated accessory/full-screen overlay policy and restores regular policy for the main window
- [x] REVIEW_ONLY: policy reassertion never calls set_focus
- [x] REVIEW_ONLY: quick mode accepts Finder interaction without forcing focus or applying an NSPanel-only style
- [x] ENFORCED_BY_TEST: quick_window_presence_policy_is_fullscreen_and_finder_drop_capable
- [x] ENFORCED_BY_TEST: quick_window_keeps_standard_appkit_input_contract
- [x] ENFORCED_BY_TEST: quick_window_accepts_inactive_finder_interaction_without_forcing_focus
- [x] REVIEW_ONLY: transparent native input area expands only for visible speech or queue details
- [x] ENFORCED_BY_TEST: compact_footprint_preserves_character_anchor
- [x] ENFORCED_BY_TEST: expanded_footprint_is_clamped_to_visible_work_area
- [x] ENFORCED_BY_TEST: warm_compact_window_uses_the_expanded_cursor_anchor
- [x] FRONTEND_TEST: transparent area collapses when speech is hidden
- [x] FRONTEND_TEST: speech gets the expanded presentation area only while visible
- [x] FRONTEND_TEST: resize-generated pointer leave cannot immediately collapse speech
- [x] FRONTEND_TEST: payload_held_sprite_normalizes_silhouette_without_resizing_character_frame
- [x] Normalize the package-held silhouette inside the unchanged 96 x 128 character frame.
- [x] Reduce the transparent native mouse target through compact and expanded footprints.
- [x] Raise the quick window through the isolated macOS adapter.
- [x] Reassert presence after active-Space transitions without polling.
- [x] HISTORICAL_RUNTIME_TEST (superseded by v0.8): packaged app remained an on-screen layer-1000 window while Safari was frontmost in full screen.

## Session Close Lifecycle v0.5

- [x] DOCUMENTED_ONLY: closing an idle courier discards its queued in-memory order
- [x] REVIEW_ONLY: closing never cancels or discards a copy or delivery operation already in progress
- [x] FRONTEND_TEST: closing_discards_idle_session_but_preserves_active_work
- [x] FRONTEND_TEST: closing_idle_courier_resets_collection_before_window_close
- [x] FRONTEND_TEST: closing_active_courier_preserves_work_before_window_close
- [x] Invoke the backend collection reset before closing an idle quick window.

## Package Count Clarity v0.6

- [x] DOCUMENTED_ONLY: the ready queue summary distinguishes packaged projects from requested projects
- [x] FRONTEND_TEST: ready_queue_summary_counts_packaged_projects_against_requested_projects
- [x] Show ready collections as packaged count versus requested count.

## Repeatable Finder Intake v0.7

- [x] DOCUMENTED_ONLY: inbound Finder intake and outbound parcel handoff have separate native owners
- [x] REVIEW_ONLY: Tauri/Wry is the sole inbound Finder drop owner
- [x] ENFORCED_BY_TEST: tauri_wry_is_the_only_inbound_finder_drop_owner
- [x] ENFORCED_BY_TEST: new_order_reset_leaves_native_drop_registration_untouched
- [x] ENFORCED_BY_TEST: tauri_fallback_drop_uses_the_shared_finder_route
- [x] ENFORCED_BY_TEST: finder_intake_accepts_only_regular_als_files
- [x] ENFORCED_BY_TEST: finder_intake_rejects_symlinked_als_files
- [x] Route Tauri/Wry Finder drops through the shared ALS validation.
- [x] Remove the competing AppKit inbound destination and rearm lifecycle.
- [ ] RUNTIME_TEST: after one completed native handoff and `Nowe zlecenie`, dropping a copied ALS on the courier creates a fresh one-item order

## Finder-Compatible Overlay And Startup Lifecycle v0.8

- [x] DOCUMENTED_ONLY: quick mode uses a Finder-compatible floating overlay policy and restores regular policy for the main window
- [x] DOCUMENTED_ONLY: cold Open With requests are buffered until setup completes
- [x] REVIEW_ONLY: production does not include the Lab panic hook or drop logger
- [x] REVIEW_ONLY: Tauri/Wry remains the sole inbound Finder-drop owner
- [x] ENFORCED_BY_TEST: startup_open_urls_are_deferred_until_runtime_is_ready
- [x] ENFORCED_BY_TEST: startup_completion_precedes_normal_window_schedule
- [x] ENFORCED_BY_TEST: quick_window_presence_policy_is_fullscreen_and_finder_drop_capable
- [ ] RUNTIME_TEST: five cold Open With launches preserve the requested ALS without a crash
- [ ] RUNTIME_TEST: five Finder drops after Safari transitions are accepted exactly once
- [x] RUNTIME_TEST: full-screen Safari keeps the Courier visible and one subsequent Finder drop is accepted exactly once
- [ ] RUNTIME_TEST: handoff then `Nowe zlecenie` accepts a fresh one-item order
- [ ] RUNTIME_TEST: outbound parcel drag and Regular main-window policy do not regress
- [x] RUNTIME_TEST: one packaged cold Open With launch preserves the requested ALS in the visible queue
- [x] RUNTIME_TEST: `Nowe zlecenie` followed by one Finder drop creates a fresh one-item order
- [x] RUNTIME_TEST: ordinary launch still opens the Regular main application window
