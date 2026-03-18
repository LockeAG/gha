use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.loading && app.runs.is_empty() {
        let spinner = theme::SPINNER_FRAMES[app.spinner_frame];
        let loading = Paragraph::new(Line::from(vec![
            Span::styled(spinner, Style::default().fg(theme::RUNNING_COLOR)),
            Span::raw(" Fetching workflow runs..."),
        ]))
        .centered()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        );
        frame.render_widget(loading, area);
        return;
    }

    if app.filtered_runs.is_empty() {
        let msg = if app.runs.is_empty() {
            "No workflow runs found"
        } else {
            "No runs match current filter"
        };
        let empty = Paragraph::new(msg)
            .centered()
            .style(Style::default().fg(theme::DIM_FG))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER_COLOR)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(" "),
        Cell::from("Repo"),
        Cell::from("Workflow"),
        Cell::from("Branch"),
        Cell::from("Event"),
        Cell::from("Age"),
        Cell::from("Actor"),
    ])
    .style(
        Style::default()
            .fg(theme::DIM_FG)
            .add_modifier(Modifier::BOLD),
    )
    .height(1);

    let rows: Vec<Row> = app
        .filtered_runs
        .iter()
        .map(|&idx| {
            let run = &app.runs[idx];
            let (icon, color) = theme::status_icon(run.status, run.conclusion, app.spinner_frame);

            Row::new(vec![
                Cell::from(Span::styled(icon, Style::default().fg(color))),
                Cell::from(Span::styled(
                    short_repo(&run.repository.full_name),
                    Style::default().fg(theme::HEADER_FG),
                )),
                Cell::from(Span::styled(
                    run.name.as_deref().unwrap_or("\u{2014}"),
                    Style::default().fg(color),
                )),
                Cell::from(Span::styled(
                    run.head_branch.as_deref().unwrap_or("\u{2014}"),
                    Style::default().fg(theme::DIM_FG),
                )),
                Cell::from(Span::styled(
                    run.event.as_str(),
                    Style::default().fg(theme::DIM_FG),
                )),
                Cell::from(Span::styled(
                    format_age(run.updated_at),
                    Style::default().fg(theme::DIM_FG),
                )),
                Cell::from(Span::styled(
                    run.actor.login.as_str(),
                    Style::default().fg(theme::DIM_FG),
                )),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Percentage(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(theme::SELECTED_BG))
        .highlight_symbol("\u{25B8} ")
        .block(Block::default().borders(Borders::NONE));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn short_repo(full_name: &str) -> &str {
    full_name.split('/').last().unwrap_or(full_name)
}

fn format_age(time: chrono::DateTime<chrono::Utc>) -> String {
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
