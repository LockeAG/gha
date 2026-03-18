use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let th = theme::t();
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
            let row_bg = if i % 2 == 0 { th.bg } else { th.alt_row_bg };

            let check = if is_explicit {
                Span::styled("\u{25CF} ", Style::default().fg(th.queued))
            } else if is_watched {
                Span::styled("\u{25C9} ", Style::default().fg(th.success))
            } else {
                Span::styled("\u{25CB} ", Style::default().fg(th.dim_fg))
            };

            let name_color = if is_watched || is_explicit { th.header_fg } else { th.dim_fg };

            let age = repo
                .pushed_at
                .map(|t| format!("{} ago", theme::format_relative_time(t)))
                .unwrap_or_else(|| "\u{2014}".to_string());

            let desc = repo.description.as_deref().unwrap_or("").chars().take(40).collect::<String>();
            let archived_tag = if repo.archived { " [archived]" } else { "" };

            Row::new(vec![
                Cell::from(check),
                Cell::from(Span::styled(repo.full_name.as_str(), Style::default().fg(name_color))),
                Cell::from(Span::styled(age, Style::default().fg(th.dim_fg))),
                Cell::from(Span::styled(format!("{desc}{archived_tag}"), Style::default().fg(th.dim_fg))),
            ])
            .style(Style::default().bg(row_bg))
        })
        .collect();

    let widths = [
        Constraint::Length(2), Constraint::Fill(3),
        Constraint::Length(10), Constraint::Fill(4),
    ];

    let legend = Line::from(vec![
        Span::styled(" \u{25C9}", Style::default().fg(th.success)),
        Span::styled(" watched ", Style::default().fg(th.dim_fg)),
        Span::styled("\u{25CF}", Style::default().fg(th.queued)),
        Span::styled(" pinned ", Style::default().fg(th.dim_fg)),
        Span::styled("\u{25CB}", Style::default().fg(th.dim_fg)),
        Span::styled(" off ", Style::default().fg(th.dim_fg)),
    ]);

    let table = Table::new(rows, widths)
        .row_highlight_style(Style::default().bg(th.selected_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol("\u{25B8} ")
        .block(
            Block::default()
                .title(title)
                .title_bottom(legend)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title_style(Style::default().fg(th.header_fg))
                .style(Style::default().bg(th.bg)),
        );

    frame.render_stateful_widget(table, area, &mut app.picker_state);
}
