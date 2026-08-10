use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeWindowLevel {
    ScreenSaver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowPresenceContract {
    always_on_top: bool,
    join_all_spaces: bool,
    full_screen_auxiliary: bool,
    stationary_across_spaces: bool,
    join_all_applications: bool,
    accessory_application: bool,
    front_without_focus: bool,
    native_level: NativeWindowLevel,
}

const fn presence_contract() -> WindowPresenceContract {
    WindowPresenceContract {
        always_on_top: true,
        join_all_spaces: true,
        full_screen_auxiliary: true,
        stationary_across_spaces: true,
        join_all_applications: true,
        accessory_application: true,
        front_without_focus: true,
        native_level: NativeWindowLevel::ScreenSaver,
    }
}

pub(crate) fn apply(app: &AppHandle, window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let policy = presence_contract();
    window.set_always_on_top(policy.always_on_top)?;
    apply_platform_policy(app, window, policy)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn restore_regular_application_policy(_app: &AppHandle) {}

#[cfg(target_os = "macos")]
pub(crate) fn restore_regular_application_policy(app: &AppHandle) {
    let _ = app.run_on_main_thread(macos_presence::restore_regular_application_policy);
}

#[cfg(not(target_os = "macos"))]
fn apply_platform_policy(
    _app: &AppHandle,
    _window: &tauri::WebviewWindow,
    _policy: WindowPresenceContract,
) -> tauri::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_platform_policy(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    policy: WindowPresenceContract,
) -> tauri::Result<()> {
    if policy.join_all_spaces {
        window.set_visible_on_all_workspaces(true)?;
    }
    let native_window = window.clone();
    let observer_app = app.clone();
    let window_label = window.label().to_string();
    app.run_on_main_thread(move || {
        apply_native_policy(&native_window, policy);
        install_space_observer(observer_app, window_label);
    })
}

#[cfg(target_os = "macos")]
mod macos_presence {
    use super::{presence_contract, WindowPresenceContract};
    use objc2::rc::Retained;
    use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationPolicy, NSScreenSaverWindowLevel, NSWindow,
        NSWindowCollectionBehavior, NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification,
    };
    use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol};
    use std::cell::RefCell;
    use std::time::Duration;
    use tauri::{AppHandle, Manager};

    struct SpaceObserverIvars {
        app: AppHandle,
        window_label: String,
    }

    define_class!(
        #[unsafe(super = NSObject)]
        #[thread_kind = MainThreadOnly]
        #[ivars = SpaceObserverIvars]
        struct QuickWindowSpaceObserver;

        impl QuickWindowSpaceObserver {
            #[unsafe(method(activeSpaceDidChange:))]
            fn active_space_did_change(&self, _notification: &NSNotification) {
                let ivars = self.ivars();
                reassert_window_policy(&ivars.app, &ivars.window_label);

                // Mission Control announces the new Space while its transition is
                // still settling. Reassert once after that transition so the
                // non-activating courier joins an unrelated app's full-screen Space.
                let app = ivars.app.clone();
                let window_label = ivars.window_label.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(450));
                    let main_app = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        reassert_window_policy(&main_app, &window_label);
                    });
                });
            }
        }

        unsafe impl NSObjectProtocol for QuickWindowSpaceObserver {}
    );

    impl QuickWindowSpaceObserver {
        fn new(app: AppHandle, window_label: String, mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(SpaceObserverIvars { app, window_label });
            // SAFETY: NSObject's initializer is valid for this observer subclass.
            unsafe { msg_send![super(this), init] }
        }
    }

    thread_local! {
        static SPACE_OBSERVER: RefCell<Option<Retained<QuickWindowSpaceObserver>>> =
            const { RefCell::new(None) };
    }

    fn reassert_window_policy(app: &AppHandle, window_label: &str) {
        let Some(window) = app.get_webview_window(window_label) else {
            return;
        };
        super::apply_native_policy(&window, presence_contract());
    }

    pub(super) fn install(app: AppHandle, window_label: String) {
        SPACE_OBSERVER.with(|slot| {
            if slot.borrow().is_some() {
                return;
            }
            let Some(mtm) = MainThreadMarker::new() else {
                return;
            };
            let observer = QuickWindowSpaceObserver::new(app, window_label, mtm);
            let center = NSWorkspace::sharedWorkspace().notificationCenter();
            // SAFETY: the selector is implemented by QuickWindowSpaceObserver,
            // the observer is retained for the process lifetime, and the
            // notification carries no object-specific contract.
            unsafe {
                center.addObserver_selector_name_object(
                    &observer,
                    sel!(activeSpaceDidChange:),
                    Some(NSWorkspaceActiveSpaceDidChangeNotification),
                    None,
                );
            }
            *slot.borrow_mut() = Some(observer);
        });
    }

    pub(super) fn apply(window: &tauri::WebviewWindow, policy: WindowPresenceContract) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        if policy.accessory_application {
            NSApplication::sharedApplication(mtm)
                .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }
        let Ok(raw_window) = window.ns_window() else {
            return;
        };
        // SAFETY: Tauri owns this NSWindow while the WebviewWindow handle is
        // alive, and callers run on the AppKit main thread.
        let window = unsafe { &*raw_window.cast::<NSWindow>() };
        let mut behavior = window.collectionBehavior();
        behavior.remove(
            NSWindowCollectionBehavior::MoveToActiveSpace
                | NSWindowCollectionBehavior::Managed
                | NSWindowCollectionBehavior::Transient
                | NSWindowCollectionBehavior::ParticipatesInCycle
                | NSWindowCollectionBehavior::FullScreenPrimary
                | NSWindowCollectionBehavior::FullScreenNone
                | NSWindowCollectionBehavior::Primary
                | NSWindowCollectionBehavior::Auxiliary,
        );
        if policy.join_all_spaces {
            behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
        }
        if policy.full_screen_auxiliary {
            behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
        }
        if policy.join_all_applications {
            behavior |= NSWindowCollectionBehavior::CanJoinAllApplications;
        }
        if policy.stationary_across_spaces {
            behavior |=
                NSWindowCollectionBehavior::Stationary | NSWindowCollectionBehavior::IgnoresCycle;
        }
        window.setCollectionBehavior(behavior);
        window.setHidesOnDeactivate(false);
        window.setLevel(NSScreenSaverWindowLevel);
        if policy.front_without_focus {
            window.orderFrontRegardless();
        }
    }

    pub(super) fn restore_regular_application_policy() {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        NSApplication::sharedApplication(mtm)
            .setActivationPolicy(NSApplicationActivationPolicy::Regular);
    }
}

#[cfg(target_os = "macos")]
fn apply_native_policy(window: &tauri::WebviewWindow, policy: WindowPresenceContract) {
    macos_presence::apply(window, policy);
}

#[cfg(target_os = "macos")]
fn install_space_observer(app: AppHandle, window_label: String) {
    macos_presence::install(app, window_label);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_window_presence_policy_is_fullscreen_and_finder_drop_capable() {
        let policy = presence_contract();
        assert!(policy.always_on_top);
        assert!(policy.join_all_spaces);
        assert!(policy.full_screen_auxiliary);
        assert!(policy.stationary_across_spaces);
        assert!(policy.join_all_applications);
        assert!(policy.accessory_application);
        assert!(policy.front_without_focus);
        assert_eq!(policy.native_level, NativeWindowLevel::ScreenSaver);
    }

    #[test]
    fn quick_window_keeps_standard_appkit_input_contract() {
        let source = include_str!("quick_window_policy.rs");
        let panel_only_style = ["NSWindowStyleMask", "::", "NonactivatingPanel"].concat();
        assert!(!source.contains(&panel_only_style));
    }
}
