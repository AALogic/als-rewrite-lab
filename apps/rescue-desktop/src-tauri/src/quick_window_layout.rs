use serde::Deserialize;
use tauri::{PhysicalPosition, PhysicalSize, Position, Size, WebviewWindow};

use crate::quick_copy_entry::QUICK_WINDOW_LABEL;

pub(crate) const EXPANDED_WIDTH: f64 = 360.0;
pub(crate) const EXPANDED_HEIGHT: f64 = 300.0;
pub(crate) const COMPACT_WIDTH: f64 = 232.0;
pub(crate) const COMPACT_HEIGHT: f64 = 184.0;

pub(crate) fn position_for_expanded_anchor(
    expanded_position: PhysicalPosition<i32>,
    current_size: PhysicalSize<u32>,
    scale: f64,
) -> PhysicalPosition<i32> {
    let expanded_width = (EXPANDED_WIDTH * scale).round() as i64;
    let expanded_height = (EXPANDED_HEIGHT * scale).round() as i64;
    PhysicalPosition::new(
        to_i32(i64::from(expanded_position.x) + expanded_width - i64::from(current_size.width)),
        to_i32(i64::from(expanded_position.y) + expanded_height - i64::from(current_size.height)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum QuickWindowFootprint {
    Compact,
    Expanded,
}

impl QuickWindowFootprint {
    fn logical_size(self) -> (f64, f64) {
        match self {
            Self::Compact => (COMPACT_WIDTH, COMPACT_HEIGHT),
            Self::Expanded => (EXPANDED_WIDTH, EXPANDED_HEIGHT),
        }
    }
}

#[tauri::command]
pub(crate) fn set_quick_window_footprint(
    window: WebviewWindow,
    footprint: QuickWindowFootprint,
) -> Result<(), String> {
    if window.label() != QUICK_WINDOW_LABEL {
        return Err("QUICK_WINDOW_FOOTPRINT_WRONG_WINDOW".to_string());
    }

    let scale = window.scale_factor().map_err(window_error)?;
    let current_position = window.outer_position().map_err(window_error)?;
    let current_size = window.outer_size().map_err(window_error)?;
    let (logical_width, logical_height) = footprint.logical_size();
    let target_size = PhysicalSize::new(
        (logical_width * scale).round() as u32,
        (logical_height * scale).round() as u32,
    );
    let work_area = window
        .current_monitor()
        .map_err(window_error)?
        .map(|monitor| {
            let area = monitor.work_area();
            (area.position, area.size)
        });
    let target_position =
        bottom_right_anchored_position(current_position, current_size, target_size, work_area);

    window
        .set_size(Size::Physical(target_size))
        .map_err(window_error)?;
    window
        .set_position(Position::Physical(target_position))
        .map_err(window_error)?;
    Ok(())
}

fn bottom_right_anchored_position(
    current_position: PhysicalPosition<i32>,
    current_size: PhysicalSize<u32>,
    target_size: PhysicalSize<u32>,
    work_area: Option<(PhysicalPosition<i32>, PhysicalSize<u32>)>,
) -> PhysicalPosition<i32> {
    let right = i64::from(current_position.x) + i64::from(current_size.width);
    let bottom = i64::from(current_position.y) + i64::from(current_size.height);
    let desired_x = right - i64::from(target_size.width);
    let desired_y = bottom - i64::from(target_size.height);

    let Some((area_position, area_size)) = work_area else {
        return PhysicalPosition::new(to_i32(desired_x), to_i32(desired_y));
    };
    let minimum_x = i64::from(area_position.x);
    let minimum_y = i64::from(area_position.y);
    let maximum_x = minimum_x + i64::from(area_size.width) - i64::from(target_size.width);
    let maximum_y = minimum_y + i64::from(area_size.height) - i64::from(target_size.height);
    PhysicalPosition::new(
        to_i32(desired_x.clamp(minimum_x, maximum_x.max(minimum_x))),
        to_i32(desired_y.clamp(minimum_y, maximum_y.max(minimum_y))),
    )
}

fn to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn window_error(error: impl std::fmt::Display) -> String {
    format!("QUICK_WINDOW_FOOTPRINT_FAILED: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_footprint_preserves_character_anchor() {
        let position = bottom_right_anchored_position(
            PhysicalPosition::new(100, 200),
            PhysicalSize::new(360, 300),
            PhysicalSize::new(232, 184),
            None,
        );

        assert_eq!(position, PhysicalPosition::new(228, 316));
    }

    #[test]
    fn warm_compact_window_uses_the_expanded_cursor_anchor() {
        let position = position_for_expanded_anchor(
            PhysicalPosition::new(100, 200),
            PhysicalSize::new(232, 184),
            1.0,
        );

        assert_eq!(position, PhysicalPosition::new(228, 316));
    }

    #[test]
    fn expanded_footprint_is_clamped_to_visible_work_area() {
        let position = bottom_right_anchored_position(
            PhysicalPosition::new(0, 24),
            PhysicalSize::new(232, 184),
            PhysicalSize::new(360, 300),
            Some((PhysicalPosition::new(0, 24), PhysicalSize::new(1920, 1056))),
        );

        assert_eq!(position, PhysicalPosition::new(0, 24));
    }
}
