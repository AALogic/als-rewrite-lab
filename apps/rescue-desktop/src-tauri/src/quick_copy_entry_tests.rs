use super::*;
use std::fs::File;

#[test]
fn valid_single_als_open_request_is_accepted() {
    let directory = tempfile::tempdir();
    assert!(directory.is_ok());
    let Some(directory) = directory.ok() else {
        return;
    };
    let path = directory.path().join("Set.als");
    assert!(File::create(&path).is_ok());
    let url = Url::from_file_path(&path);
    assert!(url.is_ok());
    let Some(url) = url.ok() else {
        return;
    };
    let contexts = contexts_from_urls(&[url], "macos_open_with");
    assert_eq!(contexts.len(), 1);
    let Some(context) = contexts.into_iter().next().and_then(Result::ok) else {
        return;
    };
    assert_eq!(context.source_als_path, path);
    assert_eq!(context.source_display_name, "Set.als");
    assert_eq!(context.launch_source, "macos_open_with");
}

#[test]
fn non_als_open_request_is_refused() {
    let directory = tempfile::tempdir();
    assert!(directory.is_ok());
    let Some(directory) = directory.ok() else {
        return;
    };
    let path = directory.path().join("notes.txt");
    assert!(File::create(&path).is_ok());
    let url = Url::from_file_path(&path);
    assert!(url.is_ok());
    let Some(url) = url.ok() else {
        return;
    };
    assert_eq!(
        contexts_from_urls(&[url], "macos_open_with")
            .into_iter()
            .next()
            .unwrap_or(Err("missing")),
        Err("QUICK_COPY_SOURCE_NOT_ALS")
    );
}

#[test]
fn multiple_opened_urls_are_routed_in_order() {
    let directory = tempfile::tempdir().expect("fixture");
    let first = directory.path().join("one.als");
    let second = directory.path().join("two.als");
    assert!(File::create(&first).is_ok());
    assert!(File::create(&second).is_ok());
    let parsed = [first.clone(), second.clone()]
        .iter()
        .filter_map(|path| Url::from_file_path(path).ok())
        .collect::<Vec<_>>();
    let contexts = contexts_from_urls(&parsed, "macos_open_with")
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    assert_eq!(
        contexts
            .iter()
            .map(|item| &item.source_als_path)
            .collect::<Vec<_>>(),
        [&first, &second]
    );
}

#[test]
fn warm_launch_appends_to_active_collection() {
    let state = QuickCopyEntryState::default();
    let first = QuickCopyLaunchContext {
        request_id: "first".to_string(),
        source_als_path: PathBuf::from("/tmp/first.als"),
        source_display_name: "first.als".to_string(),
        launch_source: "cold".to_string(),
    };
    let second = QuickCopyLaunchContext {
        request_id: "second".to_string(),
        source_als_path: PathBuf::from("/tmp/second.als"),
        source_display_name: "second.als".to_string(),
        launch_source: "warm".to_string(),
    };
    state.store(Some(first));
    state.store(Some(second.clone()));
    assert_eq!(state.latest(), Some(second));
    assert!(state.quick_launch_observed());
}

#[test]
fn finder_drop_reuses_quick_als_validation() {
    let directory = tempfile::tempdir().expect("fixture");
    let als = directory.path().join("drop.als");
    let other = directory.path().join("drop.txt");
    assert!(File::create(&als).is_ok());
    assert!(File::create(&other).is_ok());
    assert!(context_from_path(&als, "finder_drop").is_ok());
    assert_eq!(
        context_from_path(&other, "finder_drop"),
        Err("QUICK_COPY_SOURCE_NOT_ALS")
    );
}

#[test]
fn tauri_wry_is_the_only_inbound_finder_drop_owner() {
    let source = include_str!("lib.rs");
    assert!(
        source.contains("tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. })")
    );
    assert!(source.contains("quick_als_intake::route_finder_drop(app_handle, &paths)"));
    assert!(!source.contains("disable_drag_drop_handler"));
    assert!(!source.contains("macos_courier_drop"));
}

#[test]
fn new_order_reset_leaves_native_drop_registration_untouched() {
    let source = include_str!("courier_commands.rs");
    let reset = source
        .split("pub(crate) fn reset_courier_collection")
        .nth(1)
        .and_then(|value| value.split("pub(crate) fn intake_native_paths").next())
        .unwrap_or_default();
    assert!(reset.contains("CourierRuntimeStore::default()"));
    assert!(reset.contains("reset_universal_payload_drag(&app)"));
    assert!(!reset.contains("registerForDraggedTypes"));
    assert!(!reset.contains("unregisterDraggedTypes"));
    assert!(!reset.contains("macos_courier_drop"));
}

#[test]
fn tauri_fallback_drop_uses_the_shared_finder_route() {
    let source = include_str!("lib.rs");
    assert!(source.contains("quick_als_intake::route_finder_drop(app_handle, &paths)"));
    assert!(!source.contains("courier_commands::intake_native_paths(app_handle, &paths)"));
}

#[test]
fn quick_window_accepts_inactive_finder_interaction_without_forcing_focus() {
    let source = include_str!("quick_copy_entry.rs");
    assert!(source.contains(".accept_first_mouse(true)"));
    assert!(!source
        .contains("crate::quick_window_policy::apply(app, &window)?;\n    window.set_focus()?;"));
}

#[test]
fn startup_completion_precedes_normal_window_schedule() {
    let source = include_str!("lib.rs");
    let setup = source
        .split(".setup(|app|")
        .nth(1)
        .and_then(|value| value.split(".invoke_handler").next())
        .unwrap_or_default();
    let finish = setup.find("quick_copy_entry::finish_startup(app.handle())");
    let schedule = setup.find("quick_copy_entry::schedule_normal_main_window(app.handle())");
    assert!(matches!((finish, schedule), (Some(a), Some(b)) if a < b));
}

#[test]
fn quick_launch_context_wire_contract_is_stable() {
    let context = QuickCopyLaunchContext {
        request_id: "quick-test".to_string(),
        source_als_path: PathBuf::from("/tmp/Set.als"),
        source_display_name: "Set.als".to_string(),
        launch_source: "macos_open_with".to_string(),
    };
    let value = serde_json::to_value(context);
    assert!(value.is_ok());
    let Some(serde_json::Value::Object(object)) = value.ok() else {
        return;
    };
    let fields: std::collections::BTreeSet<_> = object.keys().cloned().collect();
    assert_eq!(
        fields,
        [
            "launch_source".to_string(),
            "request_id".to_string(),
            "source_als_path".to_string(),
            "source_display_name".to_string(),
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn cursor_relative_position_is_clamped_to_work_area() {
    let position = clamped_position(
        PhysicalPosition::new(1910.0, 1070.0),
        PhysicalPosition::new(0, 24),
        1920,
        1056,
        600.0,
        500.0,
        32.0,
    );
    assert_eq!(position, PhysicalPosition::new(1320, 580));
}
