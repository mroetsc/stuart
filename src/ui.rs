use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use serialport::{SerialPortType, UsbPortInfo};
use std::io;
use std::time::Duration;

use crate::state::{App, Screen, TerminalMode};

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
        terminal.draw(|frame| draw(app, frame))?;
        app.poll_serial();
        // drain all pending events without blocking, then do one blocking
        // poll with a short timeout so serial data still redraws promptly
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

fn draw(app: &mut App, frame: &mut Frame) {
    match app.screen {
        Screen::PortSelect => draw_port_select(app, frame),
        Screen::Terminal => draw_terminal(app, frame),
    }
}

fn draw_port_select(app: &App, frame: &mut Frame) {
    let [info_area, main_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let [list_area, port_info_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
            .areas(main_area);

    draw_terminal_info(app, frame, info_area);
    draw_port_list(app, frame, list_area);
    draw_port_info(app, frame, port_info_area);
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

fn key_span(label: &'static str) -> Span<'static> {
    Span::styled(format!(" {} ", label), Style::default().reversed().bold())
}

fn action_span(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::DarkGray))
}

fn sep_span() -> Span<'static> {
    Span::styled(" │ ", Style::default().fg(Color::DarkGray))
}

fn help_entry(key: &'static str, action: &'static str) -> [Span<'static>; 4] {
    [
        key_span(key),
        Span::raw(" "),
        action_span(action),
        sep_span(),
    ]
}

fn help_spans(entries: &[(&'static str, &'static str)]) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = entries.iter().flat_map(|(k, a)| help_entry(k, a)).collect();
    spans.pop();
    spans
}

fn draw_help_bar(frame: &mut Frame, area: Rect) {
    let spans = help_spans(&[
        ("↑↓", "select"),
        ("Enter", "open"),
        ("r", "refresh"),
        ("q", "quit"),
    ]);
    let help = Paragraph::new(Line::from(spans)).block(Block::new().borders(Borders::ALL));
    frame.render_widget(help, area);
}

fn draw_terminal_info(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::new().borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let right_spans: Vec<Span> =
        if app.connection.is_none() && app.hold && app.screen == crate::state::Screen::Terminal {
            vec![
                sep_span(),
                Span::styled(" reconnecting… ", Style::default().fg(Color::Yellow).bold()),
            ]
        } else {
            let line_count: usize = app
                .scrollback
                .iter()
                .flat_map(|l| l.split_inclusive('\n'))
                .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
                .count();
            let max_offset = line_count.saturating_sub(app.viewport_height);
            let at_top = app.scroll_offset >= max_offset && max_offset > 0;
            if at_top {
                vec![
                    sep_span(),
                    Span::styled(" scrollback TOP ", Style::default().fg(Color::DarkGray)),
                ]
            } else if app.scroll_offset > 0 {
                vec![
                    sep_span(),
                    Span::styled(
                        format!(" scrollback +{} ", app.scroll_offset),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]
            } else {
                vec![]
            }
        };

    let right_width: u16 = right_spans.iter().map(|s| s.width() as u16).sum();

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(right_width)]).areas(inner);

    let left = Paragraph::new(Line::from(
        if app.active_port.is_empty() || app.screen == crate::state::Screen::PortSelect {
            vec![Span::styled(" stuart ", Style::default().bold())
                .bg(Color::Rgb(211, 69, 21))
                .fg(Color::Gray)]
        } else {
            vec![
                Span::styled(" stuart ", Style::default().bold())
                    .bg(Color::Rgb(211, 69, 21))
                    .fg(Color::Gray),
                sep_span(),
                Span::styled(" on", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {} ", app.active_port), Style::default().bold()),
                sep_span(),
                Span::styled(format!(" {} ", app.current_baud), Style::default().bold()),
                Span::styled("baud rate", Style::default().fg(Color::DarkGray)),
            ]
        },
    ));
    frame.render_widget(left, left_area);

    if !right_spans.is_empty() {
        frame.render_widget(Paragraph::new(Line::from(right_spans)), right_area);
    }
}

fn draw_terminal(app: &mut App, frame: &mut Frame) {
    let [info_area, output_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let inner = Rect {
        x: output_area.x + 1,
        y: output_area.y + 1,
        width: output_area.width.saturating_sub(2),
        height: output_area.height.saturating_sub(2),
    };

    app.resize_parser(inner.height, inner.width);
    app.viewport_height = inner.height as usize;

    draw_terminal_info(app, frame, info_area);

    let all_lines: Vec<&str> = app
        .scrollback
        .iter()
        .flat_map(|l| l.split_inclusive('\n'))
        .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
        .collect();
    let total = all_lines.len();
    let height = inner.height as usize;
    let max_offset = total.saturating_sub(height);
    let scrolling = app.scroll_offset > 0;

    let block = Block::new().borders(Borders::ALL);
    frame.render_widget(block, output_area);

    if scrolling {
        let end = total.saturating_sub(app.scroll_offset.min(max_offset));
        let start = end.saturating_sub(height);
        let visible: Vec<&str> = all_lines[start..end].to_vec();
        let text = visible.join("\n");
        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, inner);
    } else {
        // render live vt100 screen
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
    }

    let help = match app.terminal_mode {
        TerminalMode::Insert => {
            let mut spans = vec![
                Span::styled(
                    " INSERT ",
                    Style::default().fg(Color::Black).bg(Color::Green).bold(),
                ),
                sep_span(),
            ];
            spans.extend(help_spans(&[("Ctrl+Esc", "control mode")]));
            Paragraph::new(Line::from(spans))
        }
        TerminalMode::Control => {
            let mut spans = vec![
                Span::styled(
                    " CONTROL ",
                    Style::default().fg(Color::Black).bg(Color::Blue).bold(),
                ),
                sep_span(),
            ];
            spans.extend(help_spans(&[
                ("a/i", "insert"),
                ("↑↓", "scroll"),
                ("Esc", "bottom"),
                ("f", "flush"),
                ("c", "copy"),
                ("+/-", "baud"),
                ("Del", "port select"),
                ("q", "quit"),
            ]));
            Paragraph::new(Line::from(spans))
        }
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
    match event::read()? {
        Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            modifiers,
            ..
        }) => match app.screen {
            Screen::PortSelect => handle_port_select_key(app, code),
            Screen::Terminal => handle_terminal_key(app, code, modifiers),
        },
        Event::Mouse(mouse) => {
            if matches!(app.screen, Screen::Terminal) {
                handle_mouse(app, mouse);
            }
        }
        Event::Paste(text) => {
            if matches!(app.screen, Screen::Terminal) {
                app.send_bytes(text.into_bytes());
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => app.scroll(3),
        MouseEventKind::ScrollDown => app.scroll(-3),
        _ => {}
    }
}

fn handle_port_select_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.exit = true,
        KeyCode::Char('r') => app.refresh_ports(),
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

    let key_event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(code, modifiers));
    if let Ok(t_event) = terminput_crossterm::to_terminput(key_event) {
        let mut buf = [0u8; 16];
        if let Ok(n) = t_event.encode(&mut buf, terminput::Encoding::Xterm) {
            app.send_bytes(buf[..n].to_vec());
        }
    }
}

fn handle_control_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('a') | KeyCode::Char('i') => {
            app.scroll_to_bottom();
            app.terminal_mode = TerminalMode::Insert;
        }
        KeyCode::Up => app.scroll(3),
        KeyCode::Down => app.scroll(-3),
        KeyCode::Esc => app.scroll_to_bottom(),
        KeyCode::Backspace | KeyCode::Delete => app.disconnect(),
        KeyCode::Char('f') => {
            app.flush_screen();
        }
        KeyCode::Char('c') => {
            app.copy_to_clipboard();
        }
        KeyCode::Char('+') => app.change_baud(1),
        KeyCode::Char('-') => app.change_baud(-1),
        KeyCode::Char('q') => {
            app.disconnect();
            app.exit = true;
        }
        _ => {}
    }
}
