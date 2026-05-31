use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use serialport::SerialPortInfo;
use std::io;

#[derive(Debug)]
pub struct App {
    ports: Vec<SerialPortInfo>,
    selected: usize,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        let ports = serialport::available_ports().unwrap_or_default();
        Self {
            ports,
            selected: 0,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let [list_area, help_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());

        if self.ports.is_empty() {
            let msg = Paragraph::new("No serial ports found.")
                .block(Block::new().borders(Borders::ALL).title(" stuart "));
            frame.render_widget(msg, list_area);
        } else {
            let items: Vec<ListItem> = self
                .ports
                .iter()
                .map(|p| ListItem::new(p.port_name.clone()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title(" stuart - select a port "),
                )
                .highlight_symbol("> ")
                .highlight_style(Style::new().reversed());

            let mut state = ListState::default().with_selected(Some(self.selected));
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

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Up => self.move_selection(-1),
                KeyCode::Down => self.move_selection(1),
                KeyCode::Enter => self.open_selected(),
                _ => {}
            }
        }
        Ok(())
    }

    fn move_selection(&mut self, delta: i32) {
        if self.ports.is_empty() {
            return;
        }
        let len = self.ports.len() as i32;
        self.selected = ((self.selected as i32 + delta).rem_euclid(len)) as usize;
    }

    fn open_selected(&mut self) {
        if let Some(port) = self.ports.get(self.selected) {
            // TODO: port opening
            let _ = port.port_name.clone();
            self.exit = true;
        }
    }
}
