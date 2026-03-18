use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, InputMode, QuickFilter};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![
        Span::styled(" gha ", Style::default().fg(theme::HEADER_FG).bold()),
        Span::styled("\u{2502} ", Style::default().fg(theme::BORDER_COLOR)),
    ];

    // Repos
    if app.repos.len() <= 3 {
        for (i, repo) in app.repos.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(", ", Style::default().fg(theme::DIM_FG)));
            }
            spans.push(Span::styled(
                repo.as_str(),
                Style::default().fg(theme::HEADER_FG),
            ));
        }
    } else {
        spans.push(Span::styled(
            format!("{} repos", app.repos.len()),
            Style::default().fg(theme::HEADER_FG),
        ));
    }

    spans.push(Span::styled(
        " \u{2502} ",
        Style::default().fg(theme::BORDER_COLOR),
    ));

    // Run count (M1)
    if !app.runs.is_empty() {
        let shown = app.filtered_runs.len();
        let total = app.runs.len();
        if shown == total {
            spans.push(Span::styled(
                format!("{total} runs"),
                Style::default().fg(theme::DIM_FG),
            ));
        } else {
            spans.push(Span::styled(
                format!("{shown}/{total}"),
                Style::default().fg(theme::HEADER_FG),
            ));
        }
        spans.push(Span::styled(
            " \u{2502} ",
            Style::default().fg(theme::BORDER_COLOR),
        ));
    }

    // Rate limit
    if let Some(ref rl) = app.rate_limit {
        let color = if rl.remaining < 100 {
            theme::FAILURE_COLOR
        } else if rl.remaining < 500 {
            theme::RUNNING_COLOR
        } else {
            theme::DIM_FG
        };
        spans.push(Span::styled(
            format!("API {}/{}", rl.remaining, rl.limit),
            Style::default().fg(color),
        ));
    } else {
        spans.push(Span::styled(
            "API --",
            Style::default().fg(theme::DIM_FG),
        ));
    }

    spans.push(Span::styled(
        " \u{2502} ",
        Style::default().fg(theme::BORDER_COLOR),
    ));

    // Last refresh
    if let Some(last) = app.last_refresh {
        let ago = theme::format_relative_time(last);
        spans.push(Span::styled(
            format!("\u{21BB} {ago} ago"),
            Style::default().fg(theme::DIM_FG),
        ));
    } else if app.loading {
        let spinner = theme::SPINNER_FRAMES[app.spinner_frame];
        spans.push(Span::styled(
            format!("{spinner} loading"),
            Style::default().fg(theme::RUNNING_COLOR),
        ));
    }

    spans.push(Span::styled(
        " \u{2502} ",
        Style::default().fg(theme::BORDER_COLOR),
    ));

    // Quick filter
    let (filter_label, filter_color) = match app.quick_filter {
        QuickFilter::All => ("all", theme::DIM_FG),
        QuickFilter::Failed => ("failed", theme::FAILURE_COLOR),
        QuickFilter::Running => ("running", theme::RUNNING_COLOR),
        QuickFilter::Success => ("success", theme::SUCCESS_COLOR),
    };
    spans.push(Span::styled(filter_label, Style::default().fg(filter_color)));

    // Search indicator
    if app.input_mode == InputMode::Search || !app.search_query.is_empty() {
        spans.push(Span::styled(
            " \u{2502} ",
            Style::default().fg(theme::BORDER_COLOR),
        ));
        spans.push(Span::styled(
            "/",
            Style::default().fg(theme::RUNNING_COLOR),
        ));
        spans.push(Span::styled(
            app.search_query.as_str(),
            Style::default().fg(theme::HEADER_FG),
        ));
        if app.input_mode == InputMode::Search {
            spans.push(Span::styled(
                "\u{258E}",
                Style::default().fg(theme::RUNNING_COLOR),
            ));
        }
    }

    // Error (auto-clears via app.on_tick)
    if let Some(ref err) = app.error {
        spans.push(Span::styled(
            " \u{2502} ",
            Style::default().fg(theme::BORDER_COLOR),
        ));
        spans.push(Span::styled(
            err.as_str(),
            Style::default().fg(theme::ERROR_FG),
        ));
    }

    let header = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::BORDER_COLOR))
            .style(Style::default().bg(theme::BG_COLOR)),
    );
    frame.render_widget(header, area);
}
