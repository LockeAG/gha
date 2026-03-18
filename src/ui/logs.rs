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
            // Color error/warning lines
            let style = if line.contains("error") || line.contains("Error") || line.contains("FAILED") {
                Style::default().fg(th.failure)
            } else if line.contains("warning") || line.contains("Warning") {
                Style::default().fg(th.running)
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
