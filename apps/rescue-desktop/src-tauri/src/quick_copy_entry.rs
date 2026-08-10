use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use std::time::Duration;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, Position, State, Url, WebviewUrl,
    WebviewWindowBuilder,
};

pub(crate) const QUICK_WINDOW_LABEL: &str = "quick-copy";
pub(crate) const QUICK_LAUNCH_EVENT: &str = "quick-copy-launch";
const CURSOR_OFFSET: f64 = 16.0;
const MAIN_WINDOW_DELAY: Duration = Duration::from_millis(350);

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickCopyLaunchContext {
    pub request_id: String,
    pub source_als_path: PathBuf,
    pub source_display_name: String,
    pub launch_source: String,
}

#[derive(Default)]
pub(crate) struct QuickCopyEntryState {
    latest_context: Mutex<Option<QuickCopyLaunchContext>>,
    quick_launch_observed: AtomicBool,
}

impl QuickCopyEntryState {
    fn store(&self, context: Option<QuickCopyLaunchContext>) {
        self.quick_launch_observed.store(true, Ordering::Release);
        if let Ok(mut latest) = self.latest_context.lock() {
            *latest = context;
        }
    }

    fn latest(&self) -> Option<QuickCopyLaunchContext> {
        self.latest_context
            .lock()
            .ok()
            .and_then(|latest| latest.clone())
    }

    fn quick_launch_observed(&self) -> bool {
        self.quick_launch_observed.load(Ordering::Acquire)
    }
}

#[tauri::command]
pub(crate) fn get_quick_copy_launch_context(
    state: State<'_, QuickCopyEntryState>,
) -> Option<QuickCopyLaunchContext> {
    state.latest()
}

pub(crate) fn schedule_normal_main_window(app: &AppHandle) {
    let app_handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(MAIN_WINDOW_DELAY);
        let should_show = !app_handle
            .state::<QuickCopyEntryState>()
            .quick_launch_observed();
        if should_show {
            let main_thread_handle = app_handle.clone();
            let _ = app_handle.run_on_main_thread(move || {
                show_main_window(&main_thread_handle);
            });
        }
    });
}

pub(crate) fn route_opened_urls(app: &AppHandle, urls: &[Url], launch_source: &str) {
    let paths = local_paths_from_urls(urls)
        .into_iter()
        .filter(|path| crate::quick_als_intake::is_supported_als_path(path))
        .collect::<Vec<_>>();
    let context = paths
        .first()
        .and_then(|path| context_from_path(path, launch_source).ok());
    app.state::<QuickCopyEntryState>().store(context.clone());
    let _ = show_quick_window(app);
    let _ = crate::courier_commands::intake_native_paths(app, &paths);
    let _ = app.emit_to(QUICK_WINDOW_LABEL, QUICK_LAUNCH_EVENT, context);
}

pub(crate) fn route_secondary_args(app: &AppHandle, args: &[String]) {
    let paths: Vec<_> = args
        .iter()
        .skip(1)
        .map(PathBuf::from)
        .filter(|path| path.extension().is_some())
        .collect();
    if paths.is_empty() {
        show_main_window(app);
        return;
    }
    let urls: Vec<_> = paths
        .iter()
        .filter_map(|path| Url::from_file_path(path).ok())
        .collect();
    route_opened_urls(app, &urls, "secondary_instance");
}

pub(crate) fn show_main_window(app: &AppHandle) {
    crate::quick_window_policy::restore_regular_application_policy(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
fn contexts_from_urls(
    urls: &[Url],
    launch_source: &str,
) -> Vec<Result<QuickCopyLaunchContext, &'static str>> {
    local_paths_from_urls(urls)
        .iter()
        .map(|path| context_from_path(path, launch_source))
        .collect()
}

fn local_paths_from_urls(urls: &[Url]) -> Vec<PathBuf> {
    urls.iter()
        .filter_map(|url| url.to_file_path().ok())
        .collect()
}

fn context_from_path(
    path: &Path,
    launch_source: &str,
) -> Result<QuickCopyLaunchContext, &'static str> {
    let source_display_name = crate::quick_als_intake::validate_als_path(path)?;
    Ok(QuickCopyLaunchContext {
        request_id: next_request_id(),
        source_als_path: path.to_path_buf(),
        source_display_name,
        launch_source: launch_source.to_string(),
    })
}

fn next_request_id() -> String {
    let sequence = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("quick-{}-{sequence}", std::process::id())
}

fn show_quick_window(app: &AppHandle) -> tauri::Result<()> {
    let window = if let Some(existing) = app.get_webview_window(QUICK_WINDOW_LABEL) {
        existing
    } else {
        WebviewWindowBuilder::new(
            app,
            QUICK_WINDOW_LABEL,
            WebviewUrl::App("index.html".into()),
        )
        .title("ALS Rescue Quick Copy")
        .inner_size(
            crate::quick_window_layout::EXPANDED_WIDTH,
            crate::quick_window_layout::EXPANDED_HEIGHT,
        )
        .min_inner_size(
            crate::quick_window_layout::COMPACT_WIDTH,
            crate::quick_window_layout::COMPACT_HEIGHT,
        )
        .max_inner_size(
            crate::quick_window_layout::EXPANDED_WIDTH,
            crate::quick_window_layout::EXPANDED_HEIGHT,
        )
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .accept_first_mouse(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .visible(false)
        .build()?
    };
    position_near_cursor(app, &window)?;
    window.show()?;
    crate::quick_window_policy::apply(app, &window)?;
    Ok(())
}

fn position_near_cursor(app: &AppHandle, window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)?
        .or(app.primary_monitor()?);
    let Some(monitor) = monitor else {
        return Ok(());
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let window_width = crate::quick_window_layout::EXPANDED_WIDTH * scale;
    let window_height = crate::quick_window_layout::EXPANDED_HEIGHT * scale;
    let offset = CURSOR_OFFSET * scale;
    let expanded_position = clamped_position(
        cursor,
        area.position,
        area.size.width,
        area.size.height,
        window_width,
        window_height,
        offset,
    );
    let position = crate::quick_window_layout::position_for_expanded_anchor(
        expanded_position,
        window.outer_size()?,
        scale,
    );
    window.set_position(Position::Physical(position))
}

fn clamped_position(
    cursor: PhysicalPosition<f64>,
    work_area_origin: PhysicalPosition<i32>,
    work_area_width: u32,
    work_area_height: u32,
    window_width: f64,
    window_height: f64,
    offset: f64,
) -> PhysicalPosition<i32> {
    let minimum_x = f64::from(work_area_origin.x);
    let minimum_y = f64::from(work_area_origin.y);
    let maximum_x = minimum_x + f64::from(work_area_width) - window_width;
    let maximum_y = minimum_y + f64::from(work_area_height) - window_height;
    let x = (cursor.x + offset).clamp(minimum_x, maximum_x.max(minimum_x));
    let y = (cursor.y + offset).clamp(minimum_y, maximum_y.max(minimum_y));
    PhysicalPosition::new(x.round() as i32, y.round() as i32)
}

#[cfg(test)]
#[path = "quick_copy_entry_tests.rs"]
mod tests;
