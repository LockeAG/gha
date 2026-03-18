pub mod dashboard;
pub mod detail;
pub mod header;
pub mod theme;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;

use crate::app::{App, InputMode, View};

const MIN_WIDTH: u16 = 50;
const MIN_HEIGHT: u16 = 8;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Fill background
    Block::default()
        .style(Style::default().bg(theme::BG_COLOR))
        .render(area, frame.buffer_mut());

    // Terminal size guard
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(frame, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Track visible content rows for PageUp/PageDown
    app.visible_rows = chunks[1].height.saturating_sub(2) as usize;

    header::render(frame, chunks[0], app);

    match app.view {
        View::Dashboard => dashboard::render(frame, chunks[1], app),
        View::Detail => detail::render(frame, chunks[1], app),
    }

    render_footer(frame, chunks[2], app);
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let msg = Paragraph::new(Line::from(vec![
        Span::styled("gha", Style::default().fg(theme::HEADER_FG)),
        Span::styled(
            format!(" needs {}x{}", MIN_WIDTH, MIN_HEIGHT),
            Style::default().fg(theme::DIM_FG),
        ),
    ]))
    .centered()
    .style(Style::default().bg(theme::BG_COLOR));
    frame.render_widget(msg, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let help = match app.input_mode {
        InputMode::Search => vec![
            Span::styled(" ESC", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" cancel ", Style::default().fg(theme::DIM_FG)),
            Span::styled("ENTER", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" confirm", Style::default().fg(theme::DIM_FG)),
        ],
        InputMode::Filter => vec![
            Span::styled(
                " FILTER ",
                Style::default()
                    .fg(theme::BG_COLOR)
                    .bg(theme::RUNNING_COLOR),
            ),
            Span::styled(" 1", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" all ", Style::default().fg(theme::DIM_FG)),
            Span::styled("2", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" fail ", Style::default().fg(theme::DIM_FG)),
            Span::styled("3", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" run ", Style::default().fg(theme::DIM_FG)),
            Span::styled("4", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" pass ", Style::default().fg(theme::DIM_FG)),
            Span::styled("ESC", Style::default().fg(theme::HEADER_FG)),
            Span::styled(" close", Style::default().fg(theme::DIM_FG)),
        ],
        InputMode::Normal => match app.view {
            View::Dashboard => vec![
                Span::styled(" j/k", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" nav ", Style::default().fg(theme::DIM_FG)),
                Span::styled("\u{21B5}", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" detail ", Style::default().fg(theme::DIM_FG)),
                Span::styled("o", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" open ", Style::default().fg(theme::DIM_FG)),
                Span::styled("/", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" search ", Style::default().fg(theme::DIM_FG)),
                Span::styled("f", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" filter ", Style::default().fg(theme::DIM_FG)),
                Span::styled("r", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" refresh ", Style::default().fg(theme::DIM_FG)),
                Span::styled("^d/^u", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" page ", Style::default().fg(theme::DIM_FG)),
                Span::styled("q", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" quit", Style::default().fg(theme::DIM_FG)),
            ],
            View::Detail => vec![
                Span::styled(" j/k", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" nav ", Style::default().fg(theme::DIM_FG)),
                Span::styled("^d/^u", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" page ", Style::default().fg(theme::DIM_FG)),
                Span::styled("o", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" open ", Style::default().fg(theme::DIM_FG)),
                Span::styled("ESC", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" back ", Style::default().fg(theme::DIM_FG)),
                Span::styled("q", Style::default().fg(theme::HEADER_FG)),
                Span::styled(" quit", Style::default().fg(theme::DIM_FG)),
            ],
        },
    };

    let footer = Paragraph::new(Line::from(help))
        .style(Style::default().fg(theme::DIM_FG).bg(theme::BG_COLOR));
    frame.render_widget(footer, area);
}
