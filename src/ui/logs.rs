use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let th = theme::t();

    let title = format!(" {} ", app.log_job_name);

    if app.log_lines.is_empty() {
        let spinner = theme::SPINNER_FRAMES[app.spinner_frame];
        let loading = Paragraph::new(Line::from(vec![
            Span::styled(spinner, Style::default().fg(th.running)),
            Span::raw(" Loading logs..."),
        ]))
        .centered()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title_style(Style::default().fg(th.header_fg).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(th.bg)),
        );
        frame.render_widget(loading, area);
        return;
    }

    let visible_height = area.height.saturating_sub(2) as usize; // border top + bottom
    let total = app.log_lines.len();
    let start = app.log_scroll;
    let end = (start + visible_height).min(total);

    let lines: Vec<Line> = app.log_lines[start..end]
        .iter()
        .map(|line| {
            let style = if is_error_line(line) {
                Style::default().fg(th.failure)
            } else if is_warning_line(line) {
                Style::default().fg(th.running)
            } else if line.starts_with("##[group]") || line.starts_with("##[endgroup]") {
                Style::default().fg(th.queued).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th.dim_fg)
            };
            Line::from(Span::styled(line.as_str(), style))
        })
        .collect();

    let position = if total > 0 {
        format!(" {}/{} ", end, total)
    } else {
        String::new()
    };

    let log_view = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(title)
                .title_bottom(Line::from(Span::styled(
                    position,
                    Style::default().fg(th.dim_fg),
                )).centered())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title_style(Style::default().fg(th.header_fg).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(th.bg)),
        );

    frame.render_widget(log_view, area);
}

fn is_error_line(line: &str) -> bool {
    // GitHub Actions annotation format
    if line.starts_with("##[error]") {
        return true;
    }
    let trimmed = line.trim_start();
    // Common error prefixes (word boundary — not substring)
    trimmed.starts_with("Error:")
        || trimmed.starts_with("ERROR ")
        || trimmed.starts_with("error:")
        || trimmed.starts_with("error[")
        || trimmed.starts_with("FAILED")
        || trimmed.starts_with("FAIL ")
        || trimmed.starts_with("fatal:")
        || trimmed.starts_with("panic:")
        || trimmed.contains("): error")
        || trimmed.contains("]: error")
}

fn is_warning_line(line: &str) -> bool {
    if line.starts_with("##[warning]") {
        return true;
    }
    let trimmed = line.trim_start();
    trimmed.starts_with("Warning:")
        || trimmed.starts_with("WARNING ")
        || trimmed.starts_with("warning:")
        || trimmed.starts_with("warning[")
        || trimmed.contains("): warning")
        || trimmed.contains("]: warning")
}
