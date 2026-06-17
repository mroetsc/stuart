use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use serialport::{DataBits, FlowControl, Parity, StopBits};

use crate::serial::{InputMode, NewlineEncoding, BAUD_RATES};
use crate::state::{App, TerminalMode};

use super::common::{help_spans, wrap_spans_to_lines};

const NAME_WIDTH: usize = 17;

#[derive(Clone, Copy)]
enum Setting {
    BaudRate,
    DataBits,
    StopBits,
    Parity,
    FlowControl,
    LocalEcho,
    InputMode,
    OutgoingNewline,
}

impl Setting {
    const ALL: &'static [Self] = &[
        Self::BaudRate,
        Self::DataBits,
        Self::StopBits,
        Self::Parity,
        Self::FlowControl,
        Self::LocalEcho,
        Self::InputMode,
        Self::OutgoingNewline,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::BaudRate => "Baud Rate",
            Self::DataBits => "Data Bits",
            Self::StopBits => "Stop Bits",
            Self::Parity => "Parity",
            Self::FlowControl => "Flow Control",
            Self::LocalEcho => "Local Echo",
            Self::InputMode => "Input Mode",
            Self::OutgoingNewline => "Outgoing Newline",
        }
    }

    fn value(self, app: &App) -> String {
        match self {
            Self::BaudRate => app.port_config.baud.to_string(),
            Self::DataBits => match app.port_config.data_bits {
                DataBits::Five => "5",
                DataBits::Six => "6",
                DataBits::Seven => "7",
                DataBits::Eight => "8",
            }
            .to_string(),
            Self::StopBits => match app.port_config.stop_bits {
                StopBits::One => "1",
                StopBits::Two => "2",
            }
            .to_string(),
            Self::Parity => match app.port_config.parity {
                Parity::None => "None",
                Parity::Odd => "Odd",
                Parity::Even => "Even",
            }
            .to_string(),
            Self::FlowControl => match app.port_config.flow_control {
                FlowControl::None => "None",
                FlowControl::Software => "Software (XON/XOFF)",
                FlowControl::Hardware => "Hardware (RTS/CTS)",
            }
            .to_string(),
            Self::LocalEcho => if app.local_echo { "On" } else { "Off" }.to_string(),
            Self::InputMode => match app.input_mode {
                InputMode::Direct => "Direct",
                InputMode::Line => "Line",
            }
            .to_string(),
            Self::OutgoingNewline => match app.outgoing_newline {
                NewlineEncoding::CR => "CR",
                NewlineEncoding::LF => "LF",
                NewlineEncoding::CRLF => "CR+LF",
            }
            .to_string(),
        }
    }

    fn cycle(self, app: &mut App, dir: i32) {
        match self {
            Self::BaudRate => {
                let current = BAUD_RATES
                    .iter()
                    .position(|&r| r == app.port_config.baud)
                    .unwrap_or(4) as i32;
                let next = (current + dir).rem_euclid(BAUD_RATES.len() as i32) as usize;
                app.port_config.baud = BAUD_RATES[next];
                app.apply_port_config();
            }
            Self::DataBits => {
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
            Self::StopBits => {
                let opts = [StopBits::One, StopBits::Two];
                let current = opts
                    .iter()
                    .position(|&s| s == app.port_config.stop_bits)
                    .unwrap_or(0) as i32;
                let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
                app.port_config.stop_bits = opts[next];
                app.apply_port_config();
            }
            Self::Parity => {
                let opts = [Parity::None, Parity::Even, Parity::Odd];
                let current = opts
                    .iter()
                    .position(|&p| p == app.port_config.parity)
                    .unwrap_or(0) as i32;
                let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
                app.port_config.parity = opts[next];
                app.apply_port_config();
            }
            Self::FlowControl => {
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
            Self::LocalEcho => {
                app.local_echo = !app.local_echo;
            }
            Self::InputMode => {
                let opts = [InputMode::Direct, InputMode::Line];
                let current = opts
                    .iter()
                    .position(|&m| m == app.input_mode)
                    .unwrap_or(0) as i32;
                let next = (current + dir).rem_euclid(opts.len() as i32) as usize;
                app.input_mode = opts[next];
                if app.input_mode == InputMode::Direct {
                    app.line_buffer.clear();
                }
            }
            Self::OutgoingNewline => {
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
        }
    }
}

pub fn draw(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let width = 66u16.min(area.width);
    let list_height = Setting::ALL.len() as u16;
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

    let name_style = Style::default().bg(Color::DarkGray);
    let value_style = Style::default().add_modifier(Modifier::BOLD);
    let plain_style = Style::default();
    let baud_input_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let items: Vec<ListItem> = Setting::ALL
        .iter()
        .map(|&setting| {
            let is_baud_editing =
                matches!(setting, Setting::BaudRate) && app.settings_baud_input.is_some();

            let spans: Vec<Span> = if is_baud_editing {
                let input = app.settings_baud_input.as_deref().unwrap_or("");
                vec![
                    Span::styled(
                        format!(" {:<width$} ", setting.label(), width = NAME_WIDTH),
                        name_style,
                    ),
                    Span::styled(": ", plain_style),
                    Span::styled(format!("{}_", input), baud_input_style),
                ]
            } else {
                vec![
                    Span::styled(
                        format!(" {:<width$} ", setting.label(), width = NAME_WIDTH),
                        name_style,
                    ),
                    Span::styled(" < ", plain_style),
                    Span::styled(setting.value(app), value_style),
                    Span::styled(" >", plain_style),
                ]
            };

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(Style::default().reversed())
        .highlight_symbol("");

    let mut state = ListState::default().with_selected(Some(app.settings_cursor));
    frame.render_stateful_widget(list, list_area, &mut state);

    if inner.height > list_area.height {
        let current = Setting::ALL[app.settings_cursor];
        let spans = if app.settings_baud_input.is_some() {
            help_spans(&[("0-9", "type"), ("Space", "apply"), ("Esc", "cancel")])
        } else if matches!(current, Setting::BaudRate) {
            help_spans(&[
                ("↑↓", "navigate"),
                ("←→", "change"),
                ("Space", "type baud"),
                ("Enter", "done"),
            ])
        } else {
            help_spans(&[
                ("↑↓", "navigate"),
                ("←→ / Space", "change"),
                ("Enter", "done"),
            ])
        };
        let lines = wrap_spans_to_lines(spans, help_area.width + 2);
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
            KeyCode::Char(' ') => {
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

    let current = Setting::ALL[app.settings_cursor];

    match code {
        KeyCode::Esc => {
            app.show_settings = false;
        }
        KeyCode::Enter => {
            app.show_settings = false;
            app.terminal_mode = TerminalMode::Insert;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.settings_cursor > 0 {
                app.settings_cursor -= 1;
            } else {
                app.settings_cursor = Setting::ALL.len() - 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.settings_cursor = (app.settings_cursor + 1) % Setting::ALL.len();
        }
        KeyCode::Left | KeyCode::Char('h') => current.cycle(app, -1),
        KeyCode::Right | KeyCode::Char('l') => current.cycle(app, 1),
        KeyCode::Char(' ') => {
            if matches!(current, Setting::BaudRate) {
                app.settings_baud_input = Some(app.port_config.baud.to_string());
            } else {
                current.cycle(app, 1);
            }
        }
        _ => {}
    }
}
