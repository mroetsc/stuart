use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use serialport::{SerialPortType, UsbPortInfo};
use std::io;
use std::time::Duration;

use crate::state::{App, Screen, TerminalMode};

pub fn run(app: &mut App, terminal: &mut DefaultTerminal) -> io::Result<()> {
    while !app.exit {
        terminal.draw(|frame| draw(app, frame))?;
        app.poll_serial();
        if event::poll(Duration::from_millis(20))? {
            handle_events(app)?;
        }
    }
    Ok(())
}

fn draw(app: &mut App, frame: &mut Frame) {
    match app.screen {
        Screen::PortSelect => draw_port_select(app, frame),
        Screen::Terminal => draw_terminal(app, frame),
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

fn draw_port_list(app: &App, frame: &mut Frame, area: Rect) {
    if app.ports.is_empty() {
        let msg = Paragraph::new("No serial ports found.")
            .block(Block::new().borders(Borders::ALL).title(" stuart "));
        frame.render_widget(msg, area);
        return;
    }

    let block_title = match &app.error {
        Some(e) => format!(" Error: {} ", e),
        None => " stuart - select a port ".to_string(),
    };

    let items: Vec<ListItem> = app
        .ports
        .iter()
        .map(|p| ListItem::new(p.port_name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::new().borders(Borders::ALL).title(block_title))
        .highlight_symbol("> ")
        .highlight_style(Style::new().reversed());

    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_port_info(app: &App, frame: &mut Frame, area: Rect) {
    let content = match app.ports.get(app.selected) {
        None => Text::from("No port selected."),
        Some(port) => port_info_text(&port.port_type),
    };

    let paragraph =
        Paragraph::new(content).block(Block::new().borders(Borders::ALL).title(" port info "));
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

fn draw_help_bar(frame: &mut Frame, area: Rect) {
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

fn draw_terminal(app: &mut App, frame: &mut Frame) {
    let [output_area, help_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());

    let inner = Rect {
        x: output_area.x + 1,
        y: output_area.y + 1,
        width: output_area.width.saturating_sub(2),
        height: output_area.height.saturating_sub(2),
    };

    app.resize_parser(inner.height, inner.width);

    let title = format!("stuart on {} ", app.active_port);
    let block = Block::new().borders(Borders::ALL).title(title);
    frame.render_widget(block, output_area);

    let screen = app.parser.screen();
    let buf = frame.buffer_mut();
    for row in 0..inner.height {
        for col in 0..inner.width {
            if let Some(cell) = screen.cell(row, col) {
                let ch = cell.contents();
                if ch.is_empty() {
                    continue;
                }
                let mut style = Style::default();

                style = style.fg(vt100_color(cell.fgcolor()));
                style = style.bg(vt100_color(cell.bgcolor()));

                if cell.bold() {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cell.italic() {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline() {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                if cell.inverse() {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                buf[(inner.x + col, inner.y + row)]
                    .set_symbol(ch)
                    .set_style(style);
            }
        }
    }

    let (crow, ccol) = screen.cursor_position();
    frame.set_cursor_position((
        (inner.x + ccol).min(inner.x + inner.width - 1),
        (inner.y + crow).min(inner.y + inner.height - 1),
    ));

    let help = match app.terminal_mode {
        TerminalMode::Insert => Paragraph::new(Line::from(vec![
            " INSERT ".reversed().bold(),
            "  Ctrl+Esc ".bold(),
            "control mode ".into(),
        ])),
        TerminalMode::Control => Paragraph::new(Line::from(vec![
            " CONTROL ".reversed().bold(),
            "  a/i ".bold(),
            "insert mode  ".into(),
            "q ".bold(),
            "disconnect & quit ".into(),
        ])),
    }
    .block(Block::new().borders(Borders::ALL));
    frame.render_widget(help, help_area);
}

fn vt100_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn handle_events(app: &mut App) -> io::Result<()> {
    if let Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        modifiers,
        ..
    }) = event::read()?
    {
        match app.screen {
            Screen::PortSelect => handle_port_select_key(app, code),
            Screen::Terminal => handle_terminal_key(app, code, modifiers),
        }
    }
    Ok(())
}

fn handle_port_select_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.exit = true,
        KeyCode::Up => app.move_selection(-1),
        KeyCode::Down => app.move_selection(1),
        KeyCode::Enter => app.open_selected(),
        _ => {}
    }
}

fn handle_terminal_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match app.terminal_mode {
        TerminalMode::Insert => handle_insert_mode(app, code, modifiers),
        TerminalMode::Control => handle_control_mode(app, code),
    }
}

fn handle_insert_mode(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    if code == KeyCode::Esc && modifiers == KeyModifiers::CONTROL {
        app.terminal_mode = TerminalMode::Control;
        return;
    }

    let bytes: Option<Vec<u8>> = match code {
        KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => Some(vec![b'\x03']),
        KeyCode::Char('z') if modifiers == KeyModifiers::CONTROL => Some(vec![b'\x1a']),
        KeyCode::Char('d') if modifiers == KeyModifiers::CONTROL => Some(vec![b'\x04']),
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            Some(c.encode_utf8(&mut buf).as_bytes().to_vec())
        }
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![b'\x08']),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Esc => Some(vec![b'\x1b']),
        _ => None,
    };

    if let Some(b) = bytes {
        app.send_bytes(b);
    }
}

fn handle_control_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('a') | KeyCode::Char('i') => {
            app.terminal_mode = TerminalMode::Insert;
        }
        KeyCode::Char('q') => {
            app.disconnect();
            app.exit = true;
        }
        _ => {}
    }
}
