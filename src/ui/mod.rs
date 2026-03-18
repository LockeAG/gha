pub mod dashboard;
pub mod detail;
pub mod header;
pub mod picker;
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
    let th = theme::t();
    let area = frame.area();

    Block::default()
        .style(Style::default().bg(th.bg))
        .render(area, frame.buffer_mut());

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

    app.visible_rows = chunks[1].height.saturating_sub(2) as usize;

    header::render(frame, chunks[0], app);

    match app.view {
        View::Dashboard => dashboard::render(frame, chunks[1], app),
        View::Detail => detail::render(frame, chunks[1], app),
        View::RepoPicker => picker::render(frame, chunks[1], app),
    }

    render_footer(frame, chunks[2], app);
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let th = theme::t();
    let msg = Paragraph::new(Line::from(vec![
        Span::styled("gha", Style::default().fg(th.header_fg)),
        Span::styled(
            format!(" needs {}x{}", MIN_WIDTH, MIN_HEIGHT),
            Style::default().fg(th.dim_fg),
        ),
    ]))
    .centered()
    .style(Style::default().bg(th.bg));
    frame.render_widget(msg, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let th = theme::t();
    let key = |s: &'static str| Span::styled(s, Style::default().fg(th.header_fg));
    let lbl = |s: &'static str| Span::styled(s, Style::default().fg(th.dim_fg));

    let help: Vec<Span> = match app.input_mode {
        InputMode::Search => vec![
            key(" ESC"), lbl(" cancel "), key("ENTER"), lbl(" confirm"),
        ],
        InputMode::Filter => vec![
            Span::styled(" FILTER ", Style::default().fg(th.bg).bg(th.running)),
            key(" 1"), lbl(" all "), key("2"), lbl(" fail "),
            key("3"), lbl(" run "), key("4"), lbl(" pass "),
            key("ESC"), lbl(" close"),
        ],
        InputMode::Normal => match app.view {
            View::Dashboard => vec![
                key(" j/k"), lbl(" nav "), key("\u{21B5}"), lbl(" detail "),
                key("o"), lbl(" open "), key("R"), lbl(" rerun "),
                key("/"), lbl(" search "), key("f"), lbl(" filter "),
                key("a"), lbl(" repos "), key("r"), lbl(" refresh "),
                key("q"), lbl(" quit"),
            ],
            View::RepoPicker => vec![
                key(" j/k"), lbl(" nav "), key("Space"), lbl(" toggle "),
                key("ESC"), lbl(" apply "), key("q"), lbl(" quit"),
            ],
            View::Detail => vec![
                key(" j/k"), lbl(" nav "), key("^d/^u"), lbl(" page "),
                key("o"), lbl(" open "), key("R"), lbl(" rerun "),
                key("ESC"), lbl(" back "), key("q"), lbl(" quit"),
            ],
        },
    };

    let footer = Paragraph::new(Line::from(help))
        .style(Style::default().fg(th.dim_fg).bg(th.bg));
    frame.render_widget(footer, area);
}
