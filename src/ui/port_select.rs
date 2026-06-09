use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use serialport::{SerialPortType, UsbPortInfo};

use crate::state::App;

use super::common::{draw_error_popup, draw_info_bar, help_bar_height, help_spans, info_bar_spans};

pub fn draw(app: &App, frame: &mut Frame) {
    let info_height = help_bar_height(info_bar_spans(app), frame.area().width).0;
    let help_height = help_bar_height(help_spans_for_bar(), frame.area().width).0;

    let [info_area, main_area, help_area] = Layout::vertical([
        Constraint::Length(info_height),
        Constraint::Min(0),
        Constraint::Length(help_height),
    ])
    .areas(frame.area());

    let [list_area, port_info_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
            .areas(main_area);

    draw_info_bar(app, frame, info_area);
    draw_port_list(app, frame, list_area);
    draw_port_info(app, frame, port_info_area);
    draw_help(frame, help_area);
    draw_error_popup(app, frame);
}

pub fn handle_key(app: &mut App, code: crossterm::event::KeyCode) {
    match code {
        crossterm::event::KeyCode::Char('q') => app.exit = true,
        crossterm::event::KeyCode::Char('r') => app.refresh_ports(),
        crossterm::event::KeyCode::Char('s') => app.show_settings = true,
        crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
            app.move_selection(-1)
        }
        crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
            app.move_selection(1)
        }
        crossterm::event::KeyCode::Enter => app.open_selected(),
        _ => {}
    }
}

fn help_spans_for_bar() -> Vec<Span<'static>> {
    help_spans(&[
        ("↑↓", "select"),
        ("Enter", "open"),
        ("s", "settings"),
        ("r", "refresh"),
        ("q", "quit"),
    ])
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let (_, lines) = help_bar_height(help_spans_for_bar(), area.width);
    let help =
        Paragraph::new(ratatui::text::Text::from(lines)).block(Block::new().borders(Borders::ALL));
    frame.render_widget(help, area);
}

fn draw_port_list(app: &App, frame: &mut Frame, area: Rect) {
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
        .block(Block::new().borders(Borders::ALL).title(" select a port "))
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
