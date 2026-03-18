use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let th = theme::t();
    let run = app
        .current_run_id
        .and_then(|id| app.runs.iter().find(|r| r.id == id));

    let title = if let Some(run) = run {
        format!(
            " {} \u{2014} {} #{} ",
            run.repository.full_name,
            run.name.as_deref().unwrap_or("workflow"),
            run.run_number,
        )
    } else {
        " Detail ".to_string()
    };

    if app.jobs.is_empty() {
        let spinner = theme::SPINNER_FRAMES[app.spinner_frame];
        let loading = Paragraph::new(Line::from(vec![
            Span::styled(spinner, Style::default().fg(th.running)),
            Span::raw(" Loading jobs..."),
        ]))
        .centered()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title_style(Style::default().fg(th.header_fg).bold())
                .style(Style::default().bg(th.bg)),
        );
        frame.render_widget(loading, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    if let Some(run) = run {
        let (icon, color) = theme::status_icon(run.status, run.conclusion, app.spinner_frame);
        let summary = Paragraph::new(Line::from(vec![
            Span::styled(format!(" {icon} "), Style::default().fg(color)),
            Span::styled(
                run.head_branch.as_deref().unwrap_or("\u{2014}"),
                Style::default().fg(th.header_fg).bold(),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(th.border)),
            Span::styled(&run.event, Style::default().fg(th.dim_fg)),
            Span::styled(" \u{2502} ", Style::default().fg(th.border)),
            Span::styled(&run.actor.login, Style::default().fg(th.dim_fg)),
            Span::styled(" \u{2502} ", Style::default().fg(th.border)),
            Span::styled(
                format!("{} ago", theme::format_relative_time(run.updated_at)),
                Style::default().fg(th.dim_fg),
            ),
        ]))
        .block(
            Block::default()
                .title(title.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title_style(Style::default().fg(th.header_fg).bold())
                .style(Style::default().bg(th.surface_bg)),
        );
        frame.render_widget(summary, chunks[0]);
    }

    let mut rows = Vec::new();
    for (job_i, job) in app.jobs.iter().enumerate() {
        let (icon, color) = theme::status_icon(job.status, job.conclusion, app.spinner_frame);
        let duration = theme::format_duration(job.started_at, job.completed_at);
        let job_bg = if job_i % 2 == 0 { th.bg } else { th.alt_row_bg };

        rows.push(
            Row::new(vec![
                Cell::from(Span::styled(format!(" {icon}"), Style::default().fg(color))),
                Cell::from(Span::styled(job.name.as_str(), Style::default().fg(color).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled(format!("{duration:>10}"), Style::default().fg(th.dim_fg))),
            ])
            .style(Style::default().bg(job_bg)),
        );

        if let Some(steps) = &job.steps {
            let step_count = steps.len();
            for (si, step) in steps.iter().enumerate() {
                let (s_icon, s_color) =
                    theme::status_icon(step.status, step.conclusion, app.spinner_frame);
                let s_duration = theme::format_duration(step.started_at, step.completed_at);
                let tree_char = if si == step_count - 1 { "\u{2514}\u{2500}" } else { "\u{251C}\u{2500}" };

                rows.push(
                    Row::new(vec![
                        Cell::from(Span::styled(format!(" {tree_char}"), Style::default().fg(th.border))),
                        Cell::from(Line::from(vec![
                            Span::styled(format!("{s_icon} "), Style::default().fg(s_color)),
                            Span::styled(step.name.as_str(), Style::default().fg(s_color)),
                        ])),
                        Cell::from(Span::styled(format!("{s_duration:>10}"), Style::default().fg(th.dim_fg))),
                    ])
                    .style(Style::default().bg(job_bg)),
                );
            }
        }
    }

    let widths = [Constraint::Length(4), Constraint::Fill(1), Constraint::Length(11)];

    let table = Table::new(rows, widths)
        .row_highlight_style(Style::default().bg(th.selected_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("\u{25B8}")
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().fg(th.border))
                .style(Style::default().bg(th.bg)),
        );

    frame.render_stateful_widget(table, chunks[1], &mut app.detail_state);
}
