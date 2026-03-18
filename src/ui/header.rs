use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, InputMode, QuickFilter};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let th = theme::t();
    let mut spans = vec![
        Span::styled(" gha ", Style::default().fg(th.header_fg).bold()),
        Span::styled("\u{2502} ", Style::default().fg(th.border)),
    ];

    if app.repos.len() <= 3 {
        for (i, repo) in app.repos.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(", ", Style::default().fg(th.dim_fg)));
            }
            spans.push(Span::styled(repo.as_str(), Style::default().fg(th.header_fg)));
        }
    } else {
        spans.push(Span::styled(
            format!("{} repos", app.repos.len()),
            Style::default().fg(th.header_fg),
        ));
    }

    spans.push(Span::styled(" \u{2502} ", Style::default().fg(th.border)));

    if !app.runs.is_empty() {
        let shown = app.filtered_runs.len();
        let total = app.runs.len();
        if shown == total {
            spans.push(Span::styled(format!("{total} runs"), Style::default().fg(th.dim_fg)));
        } else {
            spans.push(Span::styled(format!("{shown}/{total}"), Style::default().fg(th.header_fg)));
        }
        spans.push(Span::styled(" \u{2502} ", Style::default().fg(th.border)));
    }

    if let Some(ref rl) = app.rate_limit {
        let color = if rl.remaining < 100 {
            th.failure
        } else if rl.remaining < 500 {
            th.running
        } else {
            th.dim_fg
        };
        spans.push(Span::styled(format!("API {}/{}", rl.remaining, rl.limit), Style::default().fg(color)));
    } else {
        spans.push(Span::styled("API --", Style::default().fg(th.dim_fg)));
    }

    spans.push(Span::styled(" \u{2502} ", Style::default().fg(th.border)));

    if let Some(last) = app.last_refresh {
        let ago = theme::format_relative_time(last);
        spans.push(Span::styled(format!("\u{21BB} {ago} ago"), Style::default().fg(th.dim_fg)));
    } else if app.loading {
        let spinner = theme::SPINNER_FRAMES[app.spinner_frame];
        spans.push(Span::styled(format!("{spinner} loading"), Style::default().fg(th.running)));
    }

    spans.push(Span::styled(" \u{2502} ", Style::default().fg(th.border)));

    let (filter_label, filter_color) = match app.quick_filter {
        QuickFilter::All => ("all", th.dim_fg),
        QuickFilter::Failed => ("failed", th.failure),
        QuickFilter::Running => ("running", th.running),
        QuickFilter::Success => ("success", th.success),
    };
    spans.push(Span::styled(filter_label, Style::default().fg(filter_color)));

    if app.input_mode == InputMode::Search || !app.search_query.is_empty() {
        spans.push(Span::styled(" \u{2502} ", Style::default().fg(th.border)));
        spans.push(Span::styled("/", Style::default().fg(th.running)));
        spans.push(Span::styled(app.search_query.as_str(), Style::default().fg(th.header_fg)));
        if app.input_mode == InputMode::Search {
            spans.push(Span::styled("\u{258E}", Style::default().fg(th.running)));
        }
    }

    if let Some(ref err) = app.error {
        spans.push(Span::styled(" \u{2502} ", Style::default().fg(th.border)));
        spans.push(Span::styled(err.as_str(), Style::default().fg(th.error)));
    }

    let header = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(th.border))
            .style(Style::default().bg(th.bg)),
    );
    frame.render_widget(header, area);
}
