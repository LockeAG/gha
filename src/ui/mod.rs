pub mod dashboard;
pub mod detail;
pub mod header;
pub mod theme;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, InputMode, View};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    header::render(frame, chunks[0], app);

    match app.view {
        View::Dashboard => dashboard::render(frame, chunks[1], app),
        View::Detail => detail::render(frame, chunks[1], app),
    }

    render_footer(frame, chunks[2], app);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let help = match app.input_mode {
        InputMode::Search => vec![
            Span::styled(
                " ESC",
                Style::default().fg(theme::HEADER_FG),
            ),
            Span::styled(" cancel  ", Style::default().fg(theme::DIM_FG)),
            Span::styled("ENTER", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" confirm", Style::default().fg(theme::DIM_FG)),
        ],
        InputMode::Filter => vec![
            Span::styled(" 1", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" all  ", Style::default().fg(theme::DIM_FG)),
            Span::styled("2", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" fail  ", Style::default().fg(theme::DIM_FG)),
            Span::styled("3", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" run  ", Style::default().fg(theme::DIM_FG)),
            Span::styled("4", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" pass  ", Style::default().fg(theme::DIM_FG)),
            Span::styled("ESC", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" close", Style::default().fg(theme::DIM_FG)),
        ],
        InputMode::Normal => match app.view {
            View::Dashboard => vec![
                Span::styled(" j/k", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" nav  ", Style::default().fg(theme::DIM_FG)),
                Span::styled(
                    "\u{21B5}",
                    Style::default().fg(theme::HEADER_FG),
                ),
                Span::styled(" detail  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("o", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" open  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("/", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" search  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("f", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" filter  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("r", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" refresh  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("1-4", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" quick  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("q", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" quit", Style::default().fg(theme::DIM_FG)),
            ],
            View::Detail => vec![
                Span::styled(" j/k", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" nav  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("o", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" open  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("ESC", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" back  ", Style::default().fg(theme::DIM_FG)),
                Span::styled("q", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" quit", Style::default().fg(theme::DIM_FG)),
            ],
        },
    };

    let footer = Paragraph::new(Line::from(help)).style(Style::default().fg(theme::DIM_FG));
    frame.render_widget(footer, area);
}
