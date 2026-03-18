use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Style, Stylize};
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
                .title_style(Style::default().fg(theme::HEADER_FG).bold()),
        );
        frame.render_widget(loading, area);
        return;
    }

    let mut rows = Vec::new();
    for job in &app.jobs {
        let (icon, color) = theme::status_icon(job.status, job.conclusion, app.spinner_frame);
        let duration = format_duration(job.started_at, job.completed_at);

        rows.push(Row::new(vec![
            Cell::from(Span::styled(icon, Style::default().fg(color))),
            Cell::from(Span::styled(
                job.name.as_str(),
                Style::default().fg(color).bold(),
            )),
            Cell::from(Span::styled(
                duration,
                Style::default().fg(theme::DIM_FG),
            )),
        ]));

        if let Some(steps) = &job.steps {
            for step in steps {
                let (s_icon, s_color) =
                    theme::status_icon(step.status, step.conclusion, app.spinner_frame);
                let s_duration = format_duration(step.started_at, step.completed_at);

                rows.push(Row::new(vec![
                    Cell::from(Span::styled(
                        format!("  {s_icon}"),
                        Style::default().fg(s_color),
                    )),
                    Cell::from(Span::styled(
                        format!("  {}", step.name),
                        Style::default().fg(s_color),
                    )),
                    Cell::from(Span::styled(
                        s_duration,
                        Style::default().fg(theme::DIM_FG),
                    )),
                ]));
            }
        }
    }

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(75),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .row_highlight_style(Style::default().bg(theme::SELECTED_BG))
        .highlight_symbol("\u{25B8} ")
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .title_style(Style::default().fg(theme::HEADER_FG).bold()),
        );

    frame.render_stateful_widget(table, area, &mut app.detail_state);
}

fn format_duration(
    started: Option<chrono::DateTime<chrono::Utc>>,
    completed: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (started, completed) {
        (Some(start), Some(end)) => {
            let secs = (end - start).num_seconds();
            if secs < 60 {
                format!("{secs}s")
            } else if secs < 3600 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
            }
        }
        (Some(start), None) => {
            let secs = (chrono::Utc::now() - start).num_seconds();
            if secs < 60 {
                format!("{secs}s...")
            } else {
                format!("{}m {}s...", secs / 60, secs % 60)
            }
        }
        _ => "\u{2014}".to_string(),
    }
}
