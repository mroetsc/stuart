use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use serialport::{DataBits, FlowControl, Parity, StopBits};

use crate::serial::BAUD_RATES;
use crate::state::App;

use super::common::{help_spans, sep_span, wrap_spans_to_lines};

const SETTINGS: &[&str] = &[
    "Baud Rate",
    "Data Bits",
    "Stop Bits",
    "Parity",
    "Flow Control",
    "Local Echo",
    "Outgoing Newline",
];

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let width = 52u16.min(area.width);
    let list_height = SETTINGS.len() as u16;
    let help_height = 1u16;
    let height = (list_height + help_height + 3).min(area.height);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup_area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(211, 69, 21)))
        .title(Span::styled(
            " settings ",
            Style::default().fg(Color::Rgb(211, 69, 21)).bold(),
        ));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let list_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: list_height.min(inner.height),
    };
    let help_area = Rect {
        x: inner.x,
        y: inner.y + list_area.height,
        width: inner.width,
        height: inner.height.saturating_sub(list_area.height),
    };

    let items: Vec<ListItem> = SETTINGS
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let is_baud_editing = i == 0 && app.settings_baud_input.is_some();
            let value_span = if is_baud_editing {
                let input = app.settings_baud_input.as_deref().unwrap_or("");
                Span::styled(
                    format!("{}_", input),
                    Style::default().fg(Color::Yellow).bold(),
                )
            } else {
                Span::styled(setting_value(app, i), Style::default().bold())
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<13}", name),
                    Style::default().fg(Color::DarkGray),
                ),
                sep_span(),
                Span::raw(" "),
                value_span,
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().reversed())
        .highlight_symbol("");

    let mut state = ListState::default().with_selected(Some(app.settings_cursor));
    frame.render_stateful_widget(list, list_area, &mut state);

    if inner.height > list_area.height {
        let help_spans = if app.settings_baud_input.is_some() {
            help_spans(&[("0-9", "type"), ("Enter", "apply"), ("Esc", "cancel")])
        } else {
            help_spans(&[("↑↓", "navigate"), ("←→", "cycle"), ("Esc", "close")])
        };

        let lines = wrap_spans_to_lines(help_spans, help_area.width + 2);
        frame.render_widget(Paragraph::new(ratatui::text::Text::from(lines)), help_area);
    }
}

pub fn handle_key(app: &mut App, code: KeyCode, _modifiers: KeyModifiers) {
    if app.settings_baud_input.is_some() {
        match code {
            KeyCode::Esc => {
                app.settings_baud_input = None;
            }
            KeyCode::Enter => {
                let baud = app
                    .settings_baud_input
                    .as_deref()
                    .unwrap_or("")
                    .parse::<u32>()
                    .unwrap_or(0);
                if baud > 0 {
                    app.port_config.baud = baud;
                    app.apply_port_config();
                }
                app.settings_baud_input = None;
            }
            KeyCode::Backspace => {
                if let Some(ref mut s) = app.settings_baud_input {
                    s.pop();
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(ref mut s) = app.settings_baud_input {
                    s.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Esc => {
            app.show_settings = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.settings_cursor > 0 {
                app.settings_cursor -= 1;
            } else {
                app.settings_cursor = SETTINGS.len() - 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.settings_cursor = (app.settings_cursor + 1) % SETTINGS.len();
        }
        KeyCode::Left | KeyCode::Char('h') => cycle_setting(app, -1),
        KeyCode::Right | KeyCode::Char('l') => cycle_setting(app, 1),
        KeyCode::Enter => {
            if app.settings_cursor == 0 {
                app.settings_baud_input = Some(app.port_config.baud.to_string());
            } else {
                cycle_setting(app, 1);
            }
        }
        _ => {}
    }
}

fn setting_value(app: &App, index: usize) -> String {
    match index {
        0 => app.port_config.baud.to_string(),
        1 => match app.port_config.data_bits {
            DataBits::Five => "5".to_string(),
            DataBits::Six => "6".to_string(),
            DataBits::Seven => "7".to_string(),
            DataBits::Eight => "8".to_string(),
        },
        2 => match app.port_config.stop_bits {
            StopBits::One => "1".to_string(),
            StopBits::Two => "2".to_string(),
        },
        3 => match app.port_config.parity {
            Parity::None => "None".to_string(),
            Parity::Odd => "Odd".to_string(),
            Parity::Even => "Even".to_string(),
        },
        4 => match app.port_config.flow_control {
            FlowControl::None => "None".to_string(),
            FlowControl::Software => "Software (XON/XOFF)".to_string(),
            FlowControl::Hardware => "Hardware (RTS/CTS)".to_string(),
        },
        5 => {
            if app.local_echo {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        }
        6 => {
            use crate::serial::NewlineEncoding;
            match app.outgoing_newline {
                NewlineEncoding::CR => "CR".to_string(),
                NewlineEncoding::LF => "LF".to_string(),
                NewlineEncoding::CRLF => "CR+LF".to_string(),
            }
        }
        _ => String::new(),
    }
}

fn cycle_setting(app: &mut App, dir: i32) {
    match app.settings_cursor {
        0 => {
            let current = BAUD_RATES
                .iter()
                .position(|&r| r == app.port_config.baud)
                .unwrap_or(4) as i32;
            let next = (current + dir).rem_euclid(BAUD_RATES.len() as i32) as usize;
            app.port_config.baud = BAUD_RATES[next];
            app.apply_port_config();
        }
        1 => {
            let opts = [
                DataBits::Five,
                DataBits::Six,
                DataBits::Seven,
                DataBits::Eight,
            ];
            let current = opts
                .iter()
                .position(|&d| d == app.port_config.data_bits)
                .unwrap_or(3) as i32;
            let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
            app.port_config.data_bits = opts[next];
            app.apply_port_config();
        }
        2 => {
            let opts = [StopBits::One, StopBits::Two];
            let current = opts
                .iter()
                .position(|&s| s == app.port_config.stop_bits)
                .unwrap_or(0) as i32;
            let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
            app.port_config.stop_bits = opts[next];
            app.apply_port_config();
        }
        3 => {
            let opts = [Parity::None, Parity::Even, Parity::Odd];
            let current = opts
                .iter()
                .position(|&p| p == app.port_config.parity)
                .unwrap_or(0) as i32;
            let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
            app.port_config.parity = opts[next];
            app.apply_port_config();
        }
        4 => {
            let opts = [
                FlowControl::None,
                FlowControl::Software,
                FlowControl::Hardware,
            ];
            let current = opts
                .iter()
                .position(|&f| f == app.port_config.flow_control)
                .unwrap_or(0) as i32;
            let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
            app.port_config.flow_control = opts[next];
            app.apply_port_config();
        }
        5 => {
            app.local_echo = !app.local_echo;
        }
        6 => {
            use crate::serial::NewlineEncoding;
            let opts = [
                NewlineEncoding::CR,
                NewlineEncoding::LF,
                NewlineEncoding::CRLF,
            ];
            let current = opts
                .iter()
                .position(|&n| n == app.outgoing_newline)
                .unwrap_or(2) as i32;
            let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
            app.outgoing_newline = opts[next];
        }
        _ => {}
    }
}
