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
        .style(Style::default().fg(theme::DIM_FG).bg(theme::BG_COLOR))
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
            .style(Style::default().fg(theme::DIM_FG).bg(theme::BG_COLOR))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER_COLOR)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let wide = area.width >= 100;
    let multi_org = has_multiple_orgs(app);

    let (header_cells, widths): (Vec<Cell>, Vec<Constraint>) = if wide {
        // Wide: status, repo, workflow, branch, event, age, #
        (
            vec![
                Cell::from(" "),
                Cell::from("Repo"),
                Cell::from("Workflow"),
                Cell::from("Branch"),
                Cell::from("Event"),
                Cell::from("  Age"),
                Cell::from("    #"),
            ],
            vec![
                Constraint::Length(2),
                Constraint::Fill(2),
                Constraint::Fill(3),
                Constraint::Fill(2),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Length(6),
            ],
        )
    } else {
        // Compact: status, repo, workflow, branch, age, #
        (
            vec![
                Cell::from(" "),
                Cell::from("Repo"),
                Cell::from("Workflow"),
                Cell::from("Branch"),
                Cell::from("  Age"),
                Cell::from("    #"),
            ],
            vec![
                Constraint::Length(2),
                Constraint::Fill(2),
                Constraint::Fill(3),
                Constraint::Fill(2),
                Constraint::Length(6),
                Constraint::Length(6),
            ],
        )
    };

    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(theme::DIM_FG)
                .bg(theme::BG_COLOR)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows: Vec<Row> = app
        .filtered_runs
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let run = &app.runs[idx];
            let (icon, color) = theme::status_icon(run.status, run.conclusion, app.spinner_frame);
            let row_bg = if i % 2 == 0 {
                theme::BG_COLOR
            } else {
                theme::ALT_ROW_BG
            };

            let repo_display = if multi_org {
                run.repository.full_name.as_str()
            } else {
                short_repo(&run.repository.full_name)
            };

            let age = theme::format_relative_time(run.updated_at);
            let run_num = format!("#{}", run.run_number);

            let mut cells = vec![
                Cell::from(Span::styled(icon, Style::default().fg(color))),
                Cell::from(Span::styled(
                    repo_display,
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
            ];

            if wide {
                cells.push(Cell::from(Span::styled(
                    short_event(&run.event),
                    Style::default().fg(theme::DIM_FG),
                )));
            }

            cells.push(Cell::from(Span::styled(
                format!("{age:>5}"),
                Style::default().fg(theme::DIM_FG),
            )));
            cells.push(Cell::from(Span::styled(
                format!("{run_num:>5}"),
                Style::default().fg(theme::DIM_FG),
            )));

            Row::new(cells).style(Style::default().bg(row_bg))
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(
            Style::default()
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{25B8} ")
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(theme::BG_COLOR)),
        );

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn short_repo(full_name: &str) -> &str {
    full_name.split('/').last().unwrap_or(full_name)
}

fn short_event(event: &str) -> &str {
    match event {
        "pull_request" => "pr",
        "pull_request_target" => "pr_target",
        "workflow_dispatch" => "dispatch",
        "workflow_run" => "wf_run",
        "repository_dispatch" => "repo_disp",
        "merge_group" => "merge",
        other => other,
    }
}

fn has_multiple_orgs(app: &App) -> bool {
    if app.repos.len() <= 1 {
        return false;
    }
    let first_org = app.repos[0].split('/').next();
    app.repos
        .iter()
        .any(|r| r.split('/').next() != first_org)
}
