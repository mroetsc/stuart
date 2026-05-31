use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use serialport::{SerialPortType, UsbPortInfo};
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
    let [main_area, help_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());

    let [list_area, info_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
            .areas(main_area);

    draw_port_list(app, frame, list_area);
    draw_port_info(app, frame, info_area);
    draw_help_bar(frame, help_area);
}

fn draw_port_list(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    if app.ports.is_empty() {
        let msg = Paragraph::new("No serial ports found.")
            .block(Block::new().borders(Borders::ALL).title(" stuart "));
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
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

    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_port_info(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::new().borders(Borders::ALL).title(" port info ");

    let content = match app.ports.get(app.selected) {
        None => Text::from("No port selected."),
        Some(port) => port_info_text(&port.port_type),
    };

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn port_info_text(port_type: &SerialPortType) -> Text<'_> {
    match port_type {
        SerialPortType::UsbPort(info) => usb_info_text(info),
        SerialPortType::BluetoothPort => Text::from("Type: Bluetooth"),
        SerialPortType::PciPort => Text::from("Type: PCI"),
        SerialPortType::Unknown => Text::from("Type: Unknown"),
    }
}

fn usb_info_text(info: &UsbPortInfo) -> Text<'_> {
    let lines = vec![
        Line::from(vec!["Type:         ".bold(), "USB".into()]),
        Line::from(vec![
            "VID:          ".bold(),
            format!("{:#06x}", info.vid).into(),
        ]),
        Line::from(vec![
            "PID:          ".bold(),
            format!("{:#06x}", info.pid).into(),
        ]),
        Line::from(vec![
            "Manufacturer: ".bold(),
            info.manufacturer
                .clone()
                .unwrap_or_else(|| "-".to_string())
                .into(),
        ]),
        Line::from(vec![
            "Product:      ".bold(),
            info.product
                .clone()
                .unwrap_or_else(|| "-".to_string())
                .into(),
        ]),
        Line::from(vec![
            "Serial:       ".bold(),
            info.serial_number
                .clone()
                .unwrap_or_else(|| "-".to_string())
                .into(),
        ]),
    ];
    Text::from(lines)
}

fn draw_help_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help = Paragraph::new(Line::from(vec![
        " ↑↓ ".bold(),
        "select  ".into(),
        "Enter ".bold(),
        "open  ".into(),
        "q ".bold(),
        "quit ".into(),
    ]))
    .block(Block::new().borders(Borders::ALL));
    frame.render_widget(help, area);
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
