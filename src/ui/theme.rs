use std::sync::OnceLock;

use ratatui::style::Color;

use crate::models::{Conclusion, RunStatus};

static THEME: OnceLock<Theme> = OnceLock::new();

pub fn init(name: &str) {
    let t = match name {
        "tokyo-night" | "tn" => tokyo_night(),
        "tokyo-night-storm" | "tns" => tokyo_night_storm(),
        "catppuccin-mocha" | "mocha" => catppuccin_mocha(),
        _ => catppuccin_mocha(),
    };
    THEME.set(t).ok();
}

pub fn t() -> &'static Theme {
    THEME.get().expect("theme not initialized — call theme::init() first")
}

pub struct Theme {
    pub bg: Color,
    pub alt_row_bg: Color,
    pub surface_bg: Color,
    pub header_fg: Color,
    pub selected_bg: Color,
    pub border: Color,
    pub dim_fg: Color,
    pub error: Color,
    pub success: Color,
    pub failure: Color,
    pub running: Color,
    pub queued: Color,
    pub cancelled: Color,
    pub fzf_colors: &'static str,
}

fn catppuccin_mocha() -> Theme {
    Theme {
        bg: Color::Rgb(30, 30, 46),        // #1e1e2e
        alt_row_bg: Color::Rgb(49, 50, 68),    // #313244
        surface_bg: Color::Rgb(36, 39, 58),     // #24273a
        header_fg: Color::Rgb(205, 214, 244),   // #cdd6f4
        selected_bg: Color::Rgb(69, 71, 90),    // #45475a
        border: Color::Rgb(88, 91, 112),         // #585b70
        dim_fg: Color::Rgb(147, 153, 178),       // #9399b2
        error: Color::Rgb(243, 139, 168),        // #f38ba8
        success: Color::Rgb(166, 227, 161),      // #a6e3a1
        failure: Color::Rgb(243, 139, 168),      // #f38ba8
        running: Color::Rgb(249, 226, 175),      // #f9e2af
        queued: Color::Rgb(137, 180, 250),       // #89b4fa
        cancelled: Color::Rgb(147, 153, 178),    // #9399b2
        fzf_colors: "bg+:#313244,fg+:#cdd6f4,hl:#f38ba8,hl+:#f38ba8,pointer:#cba6f7,info:#585b70,border:#585b70,header:#a6e3a1",
    }
}

fn tokyo_night() -> Theme {
    Theme {
        bg: Color::Rgb(26, 27, 38),         // #1a1b26
        alt_row_bg: Color::Rgb(41, 46, 66),     // #292e42
        surface_bg: Color::Rgb(22, 22, 30),      // #16161e
        header_fg: Color::Rgb(192, 202, 245),    // #c0caf5
        selected_bg: Color::Rgb(41, 46, 66),     // #292e42
        border: Color::Rgb(59, 66, 97),           // #3b4261
        dim_fg: Color::Rgb(86, 95, 137),          // #565f89
        error: Color::Rgb(247, 118, 142),         // #f7768e
        success: Color::Rgb(158, 206, 106),       // #9ece6a
        failure: Color::Rgb(247, 118, 142),       // #f7768e
        running: Color::Rgb(224, 175, 104),       // #e0af68
        queued: Color::Rgb(122, 162, 247),        // #7aa2f7
        cancelled: Color::Rgb(86, 95, 137),       // #565f89
        fzf_colors: "bg+:#292e42,fg+:#c0caf5,hl:#f7768e,hl+:#f7768e,pointer:#bb9af7,info:#3b4261,border:#3b4261,header:#9ece6a",
    }
}

fn tokyo_night_storm() -> Theme {
    Theme {
        bg: Color::Rgb(36, 40, 59),          // #24283b
        alt_row_bg: Color::Rgb(41, 46, 66),      // #292e42
        surface_bg: Color::Rgb(31, 35, 53),       // #1f2335
        header_fg: Color::Rgb(192, 202, 245),     // #c0caf5
        selected_bg: Color::Rgb(55, 60, 83),      // #373c53
        border: Color::Rgb(59, 66, 97),            // #3b4261
        dim_fg: Color::Rgb(86, 95, 137),           // #565f89
        error: Color::Rgb(247, 118, 142),          // #f7768e
        success: Color::Rgb(158, 206, 106),        // #9ece6a
        failure: Color::Rgb(247, 118, 142),        // #f7768e
        running: Color::Rgb(224, 175, 104),        // #e0af68
        queued: Color::Rgb(122, 162, 247),         // #7aa2f7
        cancelled: Color::Rgb(86, 95, 137),        // #565f89
        fzf_colors: "bg+:#292e42,fg+:#c0caf5,hl:#f7768e,hl+:#f7768e,pointer:#bb9af7,info:#3b4261,border:#3b4261,header:#9ece6a",
    }
}

pub const SPINNER_FRAMES: &[&str] = &[
    "\u{28CB}", "\u{2899}", "\u{28B9}", "\u{28B8}", "\u{28BC}", "\u{28B4}", "\u{28A6}",
    "\u{28A7}", "\u{2887}", "\u{288F}",
];

pub fn status_icon(
    status: RunStatus,
    conclusion: Option<Conclusion>,
    spinner_frame: usize,
) -> (&'static str, Color) {
    let th = t();
    match status {
        RunStatus::Completed => match conclusion {
            Some(Conclusion::Success) => ("\u{2713}", th.success),
            Some(Conclusion::Failure) => ("\u{2717}", th.failure),
            Some(Conclusion::Cancelled) => ("\u{2298}", th.cancelled),
            Some(Conclusion::Skipped) => ("\u{2298}", th.cancelled),
            Some(Conclusion::TimedOut) => ("\u{23F1}", th.failure),
            _ => ("?", th.dim_fg),
        },
        RunStatus::InProgress => (SPINNER_FRAMES[spinner_frame], th.running),
        RunStatus::Queued | RunStatus::Waiting | RunStatus::Pending | RunStatus::Requested => {
            ("\u{25F7}", th.queued)
        }
        RunStatus::Unknown => ("?", th.dim_fg),
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

