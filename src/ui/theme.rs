use ratatui::style::Color;

use crate::models::{Conclusion, RunStatus};

// Catppuccin Mocha base
pub const BG_COLOR: Color = Color::Rgb(30, 30, 46);
pub const ALT_ROW_BG: Color = Color::Rgb(49, 50, 68);
pub const SURFACE_BG: Color = Color::Rgb(36, 39, 58);

pub const HEADER_FG: Color = Color::Rgb(205, 214, 244);
pub const SELECTED_BG: Color = Color::Rgb(69, 71, 90);
pub const BORDER_COLOR: Color = Color::Rgb(88, 91, 112);
pub const DIM_FG: Color = Color::Rgb(147, 153, 178);
pub const ERROR_FG: Color = Color::Rgb(243, 139, 168);

pub const SUCCESS_COLOR: Color = Color::Rgb(166, 227, 161);
pub const FAILURE_COLOR: Color = Color::Rgb(243, 139, 168);
pub const RUNNING_COLOR: Color = Color::Rgb(249, 226, 175);
pub const QUEUED_COLOR: Color = Color::Rgb(137, 180, 250);
pub const CANCELLED_COLOR: Color = Color::Rgb(147, 153, 178);

pub const SPINNER_FRAMES: &[&str] = &[
    "\u{28CB}", "\u{2899}", "\u{28B9}", "\u{28B8}", "\u{28BC}", "\u{28B4}", "\u{28A6}",
    "\u{28A7}", "\u{2887}", "\u{288F}",
];

pub fn status_icon(
    status: RunStatus,
    conclusion: Option<Conclusion>,
    spinner_frame: usize,
) -> (&'static str, Color) {
    match status {
        RunStatus::Completed => match conclusion {
            Some(Conclusion::Success) => ("\u{2713}", SUCCESS_COLOR),
            Some(Conclusion::Failure) => ("\u{2717}", FAILURE_COLOR),
            Some(Conclusion::Cancelled) => ("\u{2298}", CANCELLED_COLOR),
            Some(Conclusion::Skipped) => ("\u{2298}", CANCELLED_COLOR),
            Some(Conclusion::TimedOut) => ("\u{23F1}", FAILURE_COLOR),
            _ => ("?", DIM_FG),
        },
        RunStatus::InProgress => (SPINNER_FRAMES[spinner_frame], RUNNING_COLOR),
        RunStatus::Queued | RunStatus::Waiting | RunStatus::Pending | RunStatus::Requested => {
            ("\u{25F7}", QUEUED_COLOR)
        }
        RunStatus::Unknown => ("?", DIM_FG),
    }
}

pub fn format_relative_time(time: chrono::DateTime<chrono::Utc>) -> String {
    let diff = chrono::Utc::now() - time;
    if diff.num_seconds() < 60 {
        format!("{}s", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h", diff.num_hours())
    } else {
        format!("{}d", diff.num_days())
    }
}
