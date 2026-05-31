use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use std::io;

use crate::state::{App, Screen};

pub fn run(app: &mut App, terminal: &mut DefaultTerminal) -> io::Result<()> {
    while !app.exit {
        terminal.draw(|frame| draw(app, frame))?;
        handle_events(app)?;
    }
    Ok(())
}

fn draw(app: &App, frame: &mut Frame) {
    match app.screen {
        Screen::PortSelect => draw_port_select(app, frame),
        Screen::Terminal => draw_terminal(frame),
    }
}

fn draw_port_select(app: &App, frame: &mut Frame) {
    let [list_area, help_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());

    if app.ports.is_empty() {
        let msg = Paragraph::new("No serial ports found.")
            .block(Block::new().borders(Borders::ALL).title(" stuart "));
        frame.render_widget(msg, list_area);
    } else {
        let items: Vec<ListItem> = app
            .ports
            .iter()
            .map(|p| ListItem::new(p.port_name.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .title(" stuart — select a port "),
            )
            .highlight_symbol("> ")
            .highlight_style(Style::new().reversed());

        let mut state = ListState::default().with_selected(Some(app.selected));
        frame.render_stateful_widget(list, list_area, &mut state);
    }

    let help = Paragraph::new(Line::from(vec![
        " ↑↓ ".bold(),
        "select  ".into(),
        "Enter ".bold(),
        "open  ".into(),
        "q ".bold(),
        "quit ".into(),
    ]))
    .block(Block::new().borders(Borders::ALL));
    frame.render_widget(help, help_area);
}

fn draw_terminal(frame: &mut Frame) {
    todo!("terminal")
}

fn handle_events(app: &mut App) -> io::Result<()> {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    }) = event::read()?
    {
        match app.screen {
            Screen::PortSelect => handle_port_select_key(app, code),
            Screen::Terminal => handle_terminal_key(app, code),
        }
    }
    Ok(())
}

fn handle_port_select_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.exit = true,
        KeyCode::Up => app.move_selection(-1),
        KeyCode::Down => app.move_selection(1),
        KeyCode::Enter => app.screen = Screen::Terminal,
        _ => {}
    }
}

fn handle_terminal_key(app: &mut App, code: KeyCode) {
    if let KeyCode::Char('q') = code {
        app.exit = true
    };
}
