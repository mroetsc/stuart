pub mod common;
pub mod port_select;
pub mod settings;
pub mod terminal;

use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyEvent, KeyEventKind,
};
use ratatui::DefaultTerminal;
use std::io;
use std::time::Duration;

use crate::state::{App, Screen};

pub fn run(app: &mut App, terminal: &mut DefaultTerminal) -> io::Result<()> {
    crossterm::execute!(std::io::stdout(), EnableMouseCapture, EnableBracketedPaste)?;
    let result = run_inner(app, terminal);
    crossterm::execute!(
        std::io::stdout(),
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    result
}

fn run_inner(app: &mut App, terminal: &mut DefaultTerminal) -> io::Result<()> {
    while !app.exit {
        terminal.draw(|frame| {
            match app.screen {
                Screen::PortSelect => port_select::draw(app, frame),
                Screen::Terminal => terminal::draw(app, frame),
            }
            if app.settings.show {
                settings::draw(app, frame);
            }
        })?;

        app.poll_serial();

        while event::poll(Duration::from_millis(0))? {
            handle_events(app)?;
            if app.exit {
                return Ok(());
            }
        }
        if event::poll(Duration::from_millis(10))? {
            handle_events(app)?;
        }
    }
    Ok(())
}

fn handle_events(app: &mut App) -> io::Result<()> {
    match event::read()? {
        Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) => {
            if app.settings.show {
                settings::handle_key(app, code, modifiers);
            } else {
                match app.screen {
                    Screen::PortSelect => port_select::handle_key(app, code),
                    Screen::Terminal => terminal::handle_key(app, code, modifiers),
                }
            }
        }
        Event::Mouse(mouse) => {
            if matches!(app.screen, Screen::Terminal) && !app.settings.show {
                terminal::handle_mouse(app, mouse);
            }
        }
        Event::Paste(text) if matches!(app.screen, Screen::Terminal) && !app.settings.show => {
            app.send_bytes(text.into_bytes());
        }
        _ => {}
    }
    Ok(())
}
