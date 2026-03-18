use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
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
            Span::styled(spinner, Style::default().fg(theme::RUNNING_COLOR)),
            Span::raw(" Loading jobs..."),
        ]))
        .centered()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .title_style(Style::default().fg(theme::HEADER_FG).bold())
                .style(Style::default().bg(theme::BG_COLOR)),
        );
        frame.render_widget(loading, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Run context summary bar
    if let Some(run) = run {
        let (icon, color) = theme::status_icon(run.status, run.conclusion, app.spinner_frame);
        let summary = Paragraph::new(Line::from(vec![
            Span::styled(format!(" {icon} "), Style::default().fg(color)),
            Span::styled(
                run.head_branch.as_deref().unwrap_or("\u{2014}"),
                Style::default().fg(theme::HEADER_FG).bold(),
            ),
            Span::styled(
                " \u{2502} ",
                Style::default().fg(theme::BORDER_COLOR),
            ),
            Span::styled(&run.event, Style::default().fg(theme::DIM_FG)),
            Span::styled(
                " \u{2502} ",
                Style::default().fg(theme::BORDER_COLOR),
            ),
            Span::styled(&run.actor.login, Style::default().fg(theme::DIM_FG)),
            Span::styled(
                " \u{2502} ",
                Style::default().fg(theme::BORDER_COLOR),
            ),
            Span::styled(
                format!("{} ago", theme::format_relative_time(run.updated_at)),
                Style::default().fg(theme::DIM_FG),
            ),
        ]))
        .block(
            Block::default()
                .title(title.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .title_style(Style::default().fg(theme::HEADER_FG).bold())
                .style(Style::default().bg(theme::SURFACE_BG)),
        );
        frame.render_widget(summary, chunks[0]);
    }

    // Job/step tree
    let mut rows = Vec::new();
    for (job_i, job) in app.jobs.iter().enumerate() {
        let (icon, color) = theme::status_icon(job.status, job.conclusion, app.spinner_frame);
        let duration = format_duration(job.started_at, job.completed_at);
        let job_bg = if job_i % 2 == 0 {
            theme::BG_COLOR
        } else {
            theme::ALT_ROW_BG
        };

        // Job header row
        rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    format!(" {icon}"),
                    Style::default().fg(color),
                )),
                Cell::from(Span::styled(
                    job.name.as_str(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    format!("{duration:>10}"),
                    Style::default().fg(theme::DIM_FG),
                )),
            ])
            .style(Style::default().bg(job_bg)),
        );

        // Steps with tree-drawing characters
        if let Some(steps) = &job.steps {
            let step_count = steps.len();
            for (si, step) in steps.iter().enumerate() {
                let (s_icon, s_color) =
                    theme::status_icon(step.status, step.conclusion, app.spinner_frame);
                let s_duration = format_duration(step.started_at, step.completed_at);

                let tree_char = if si == step_count - 1 {
                    "\u{2514}\u{2500}" // └─
                } else {
                    "\u{251C}\u{2500}" // ├─
                };

                rows.push(
                    Row::new(vec![
                        Cell::from(Span::styled(
                            format!(" {tree_char}"),
                            Style::default().fg(theme::BORDER_COLOR),
                        )),
                        Cell::from(Line::from(vec![
                            Span::styled(
                                format!("{s_icon} "),
                                Style::default().fg(s_color),
                            ),
                            Span::styled(
                                step.name.as_str(),
                                Style::default().fg(s_color),
                            ),
                        ])),
                        Cell::from(Span::styled(
                            format!("{s_duration:>10}"),
                            Style::default().fg(theme::DIM_FG),
                        )),
                    ])
                    .style(Style::default().bg(job_bg)),
                );
            }
        }
    }

    let widths = [
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(11),
    ];

    let table = Table::new(rows, widths)
        .row_highlight_style(
            Style::default()
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{25B8}")
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .style(Style::default().bg(theme::BG_COLOR)),
        );

    frame.render_stateful_widget(table, chunks[1], &mut app.detail_state);
}

fn format_duration(
    started: Option<chrono::DateTime<chrono::Utc>>,
    completed: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (started, completed) {
        (Some(start), Some(end)) => {
            let secs = (end - start).num_seconds().max(0);
            if secs < 60 {
                format!("{secs}s")
            } else if secs < 3600 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
            }
        }
        (Some(start), None) => {
            let secs = (chrono::Utc::now() - start).num_seconds().max(0);
            if secs < 60 {
                format!("{secs}s...")
            } else {
                format!("{}m {}s...", secs / 60, secs % 60)
            }
        }
        _ => "\u{2014}".to_string(),
    }
}
