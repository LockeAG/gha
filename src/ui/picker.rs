use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let total = app.all_org_repos.len();
    let watched = app.watched_set.len();
    let title = format!(" Repos ({watched} of {total} watched) ");

    let rows: Vec<Row> = app
        .all_org_repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let is_watched = app.watched_set.contains(&repo.full_name);
            let is_explicit = app.explicit_repos.contains(&repo.full_name);
            let row_bg = if i % 2 == 0 {
                theme::BG_COLOR
            } else {
                theme::ALT_ROW_BG
            };

            let check = if is_explicit {
                Span::styled("\u{25CF} ", Style::default().fg(theme::QUEUED_COLOR)) // ● pinned
            } else if is_watched {
                Span::styled("\u{25C9} ", Style::default().fg(theme::SUCCESS_COLOR)) // ◉ on
            } else {
                Span::styled("\u{25CB} ", Style::default().fg(theme::DIM_FG)) // ○ off
            };

            let name_color = if is_watched || is_explicit {
                theme::HEADER_FG
            } else {
                theme::DIM_FG
            };

            let age = repo
                .pushed_at
                .map(|t| format!("{} ago", theme::format_relative_time(t)))
                .unwrap_or_else(|| "\u{2014}".to_string());

            let desc = repo
                .description
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(40)
                .collect::<String>();

            let archived_tag = if repo.archived { " [archived]" } else { "" };

            Row::new(vec![
                Cell::from(check),
                Cell::from(Span::styled(
                    repo.full_name.as_str(),
                    Style::default().fg(name_color),
                )),
                Cell::from(Span::styled(
                    age,
                    Style::default().fg(theme::DIM_FG),
                )),
                Cell::from(Span::styled(
                    format!("{desc}{archived_tag}"),
                    Style::default().fg(theme::DIM_FG),
                )),
            ])
            .style(Style::default().bg(row_bg))
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Fill(3),
        Constraint::Length(10),
        Constraint::Fill(4),
    ];

    let table = Table::new(rows, widths)
        .row_highlight_style(
            Style::default()
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{25B8} ")
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR))
                .title_style(Style::default().fg(theme::HEADER_FG))
                .style(Style::default().bg(theme::BG_COLOR)),
        );

    frame.render_stateful_widget(table, area, &mut app.picker_state);
}
