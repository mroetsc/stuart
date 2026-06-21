use serialport::SerialPortInfo;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crate::serial::{self, Command, InputMode, NewlineEncoding, PortConfig, SerialEvent};

#[derive(Debug, PartialEq)]
pub enum Screen {
    PortSelect,
    Terminal,
}

#[derive(Debug, PartialEq)]
pub enum TerminalMode {
    Insert,
    Control,
}

pub struct ErrorEntry {
    pub message: String,
    pub count: u32,
    pub shown_at: Instant,
}

pub struct App {
    pub screen: Screen,
    pub ports: Vec<SerialPortInfo>,
    pub selected: usize,
    pub exit: bool,
    pub connection: Option<(Sender<Command>, Receiver<SerialEvent>)>,
    pub errors: Vec<ErrorEntry>,
    pub terminal_mode: TerminalMode,
    pub active_port: String,
    pub port_config: PortConfig,
    pub parser: vt100::Parser,
    pub scrollback: Vec<String>,
    pub frozen_lines: Option<Vec<String>>,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub hold: bool,
    pub reconnect_at: Option<Instant>,
    pub show_settings: bool,
    pub settings_cursor: usize,
    pub settings_baud_input: Option<String>,
    pub keyboard_enhanced: bool,
    pub local_echo: bool,
    pub input_mode: InputMode,
    pub line_buffer: String,
    pub line_history: Vec<String>,
    pub line_history_pos: Option<usize>,
    line_buffer_saved: String,
    pub outgoing_newline: NewlineEncoding,
    clipboard: Option<arboard::Clipboard>,
}

const MAX_SCROLLBACK: usize = 10000;

impl App {
    pub fn new(hold: bool, keyboard_enhanced: bool) -> Self {
        let ports = sorted_ports(serialport::available_ports().unwrap_or_default());
        Self {
            screen: Screen::PortSelect,
            ports,
            selected: 0,
            exit: false,
            connection: None,
            errors: Vec::new(),
            terminal_mode: TerminalMode::Insert,
            active_port: String::new(),
            port_config: PortConfig::default(),
            parser: vt100::Parser::new(24, 80, 0),
            scrollback: Vec::new(),
            frozen_lines: None,
            scroll_offset: 0,
            viewport_height: 24,
            hold,
            reconnect_at: None,
            show_settings: false,
            settings_cursor: 0,
            settings_baud_input: None,
            keyboard_enhanced,
            local_echo: false,
            input_mode: InputMode::Direct,
            line_buffer: String::new(),
            line_history: Vec::new(),
            line_history_pos: None,
            line_buffer_saved: String::new(),
            outgoing_newline: NewlineEncoding::CR,
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    pub fn with_port(
        port_name: &str,
        config: PortConfig,
        hold: bool,
        keyboard_enhanced: bool,
    ) -> Self {
        let (connection, errors, screen) = match serial::open(port_name, &config) {
            Ok((tx, rx)) => (Some((tx, rx)), Vec::new(), Screen::Terminal),
            Err(e) => (
                None,
                vec![ErrorEntry {
                    message: friendly_serial_error(&e.to_string()),
                    count: 1,
                    shown_at: Instant::now(),
                }],
                Screen::PortSelect,
            ),
        };
        Self {
            screen,
            ports: sorted_ports(serialport::available_ports().unwrap_or_default()),
            selected: 0,
            exit: false,
            connection,
            errors,
            terminal_mode: TerminalMode::Insert,
            active_port: port_name.to_string(),
            port_config: config,
            parser: vt100::Parser::new(24, 80, 0),
            scrollback: Vec::new(),
            frozen_lines: None,
            scroll_offset: 0,
            viewport_height: 24,
            hold,
            reconnect_at: None,
            show_settings: false,
            settings_cursor: 0,
            settings_baud_input: None,
            keyboard_enhanced,
            local_echo: false,
            input_mode: InputMode::Direct,
            line_buffer: String::new(),
            line_history: Vec::new(),
            line_history_pos: None,
            line_buffer_saved: String::new(),
            outgoing_newline: NewlineEncoding::CR,
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    pub fn apply_port_config(&mut self) {
        if self.connection.is_none() || self.active_port.is_empty() {
            return;
        }
        self.connection = None;
        std::thread::sleep(std::time::Duration::from_millis(50));
        match serial::open(&self.active_port, &self.port_config) {
            Ok((tx, rx)) => {
                self.connection = Some((tx, rx));
            }
            Err(e) => {
                self.push_error(friendly_serial_error(&e.to_string()));
            }
        }
    }

    pub fn refresh_ports(&mut self) {
        let previous = self.ports.get(self.selected).map(|p| p.port_name.clone());
        self.ports = sorted_ports(serialport::available_ports().unwrap_or_default());
        self.selected = previous
            .and_then(|name| self.ports.iter().position(|p| p.port_name == name))
            .unwrap_or(0)
            .min(self.ports.len().saturating_sub(1));
    }

    pub fn push_error(&mut self, msg: String) {
        if let Some(entry) = self.errors.iter_mut().find(|e| e.message == msg) {
            entry.count += 1;
            entry.shown_at = Instant::now();
        } else {
            self.errors.push(ErrorEntry {
                message: msg,
                count: 1,
                shown_at: Instant::now(),
            });
        }
    }

    pub fn tick_errors(&mut self) {
        self.errors
            .retain(|e| e.shown_at.elapsed() < Duration::from_secs(5));
    }

    pub fn resize_parser(&mut self, rows: u16, cols: u16) {
        self.parser.screen_mut().set_size(rows, cols);
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.ports.is_empty() {
            return;
        }
        let len = self.ports.len() as i32;
        self.selected = ((self.selected as i32 + delta).rem_euclid(len)) as usize;
    }

    pub fn open_selected(&mut self) {
        if let Some(port) = self.ports.get(self.selected) {
            match serial::open(&port.port_name, &self.port_config) {
                Ok((tx, rx)) => {
                    self.errors.clear();
                    self.parser = vt100::Parser::new(24, 80, 0);
                    self.scrollback.clear();
                    self.frozen_lines = None;
                    self.scroll_offset = 0;
                    self.active_port = port.port_name.clone();

                    self.connection = Some((tx, rx));
                    self.screen = Screen::Terminal;
                    self.terminal_mode = TerminalMode::Insert;
                }
                Err(e) => {
                    self.push_error(friendly_serial_error(&e.to_string()));
                }
            }
        }
    }

    pub fn disconnect(&mut self) {
        if let Some((tx, _)) = &self.connection {
            let _ = tx.send(Command::Disconnect);
        }
        self.connection = None;
        self.screen = Screen::PortSelect;
    }

    pub fn send_bytes(&mut self, bytes: Vec<u8>) {
        if let Some((tx, _)) = &self.connection {
            let _ = tx.send(Command::Write(bytes));
        }
    }

    pub fn send_line(&mut self) {
        let text = std::mem::take(&mut self.line_buffer);
        self.line_history_pos = None;
        self.line_buffer_saved.clear();

        if !text.is_empty() && self.line_history.last().map(String::as_str) != Some(&text) {
            self.line_history.push(text.clone());
        }

        self.scroll_to_bottom();

        if self.local_echo {
            for byte in text.bytes() {
                self.echo_local(&[byte]);
            }
            self.echo_local(&[0x0d]);
        }

        let mut bytes = text.into_bytes();
        match self.outgoing_newline {
            NewlineEncoding::CR => bytes.push(0x0d),
            NewlineEncoding::LF => bytes.push(0x0a),
            NewlineEncoding::CRLF => {
                bytes.push(0x0d);
                bytes.push(0x0a);
            }
        }
        self.send_bytes(bytes);
    }

    pub fn history_prev(&mut self) {
        if self.line_history.is_empty() {
            return;
        }
        match self.line_history_pos {
            None => {
                self.line_buffer_saved = self.line_buffer.clone();
                self.line_history_pos = Some(0);
            }
            Some(pos) if pos + 1 < self.line_history.len() => {
                self.line_history_pos = Some(pos + 1);
            }
            _ => return,
        }
        let idx = self.line_history.len() - 1 - self.line_history_pos.unwrap();
        self.line_buffer = self.line_history[idx].clone();
    }

    pub fn history_next(&mut self) {
        match self.line_history_pos {
            None => {}
            Some(0) => {
                self.line_history_pos = None;
                self.line_buffer = std::mem::take(&mut self.line_buffer_saved);
            }
            Some(pos) => {
                self.line_history_pos = Some(pos - 1);
                let idx = self.line_history.len() - pos;
                self.line_buffer = self.line_history[idx].clone();
            }
        }
    }

    pub fn echo_local(&mut self, bytes: &[u8]) {
        use crossterm::style::{Color, ResetColor, SetForegroundColor};

        const ECHO_COLOR: Color = Color::Rgb { r: 255, g: 165, b: 0 };

        let mut parser_feed: Vec<u8> = Vec::new();

        // bit hacky but works
        match bytes {
            [0x0d] => {
                // enter advances line
                parser_feed.extend_from_slice(b"\r\n");
                if let Some(last) = self.scrollback.last_mut() && !last.ends_with('\n') {
                    last.push('\n');
                }
            }
            [b @ 0x20..=0x7e] => {
                // display everything ascii in color
                let seq = format!("{}{}{}", SetForegroundColor(ECHO_COLOR), *b as char, ResetColor);
                parser_feed.extend_from_slice(seq.as_bytes());
                let ch = *b as char;
                match self.scrollback.last_mut() {
                    Some(last) if !last.ends_with('\n') => last.push(ch),
                    _ => self.scrollback.push(ch.to_string()),
                }
            }
            _ => {
                // pass anything else like arrow keys through,
                // so parser handles cursor movement
                parser_feed.extend_from_slice(bytes);
            }
        }

        if !parser_feed.is_empty() {
            self.parser.process(&parser_feed);
        }
    }

    pub fn change_baud(&mut self, delta: i32) {
        let current = crate::serial::BAUD_RATES
            .iter()
            .position(|&r| r == self.port_config.baud)
            .unwrap_or(4);
        let next =
            (current as i32 + delta).clamp(0, crate::serial::BAUD_RATES.len() as i32 - 1) as usize;
        let new_baud = crate::serial::BAUD_RATES[next];
        if new_baud == self.port_config.baud {
            return;
        }
        self.connection = None;
        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut config = self.port_config.clone();
        config.baud = new_baud;
        match serial::open(&self.active_port, &config) {
            Ok((tx, rx)) => {
                self.port_config.baud = new_baud;
                self.port_config = config;
                self.connection = Some((tx, rx));
            }
            Err(e) => {
                self.push_error(friendly_serial_error(&e.to_string()));
                self.screen = Screen::PortSelect;
            }
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        let lines: Vec<String> = self
            .scrollback
            .iter()
            .flat_map(|l| {
                l.split_inclusive('\n')
                    .map(|s| s.trim_end_matches('\n').to_string())
            })
            .collect();
        let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
        let end = lines
            .iter()
            .rposition(|l| !l.trim().is_empty())
            .map(|i| i + 1)
            .unwrap_or(0);
        let text = lines[start..end].join("\n");
        if let Some(clipboard) = &mut self.clipboard {
            let _ = clipboard.set_text(text);
        }
    }

    pub fn scroll(&mut self, delta: i32) {
        let entering_scroll = self.scroll_offset == 0 && delta > 0;
        if entering_scroll {
            self.frozen_lines = Some(self.scrollback.clone());
        }

        let lines = self.frozen_lines.as_ref().unwrap_or(&self.scrollback);
        let line_count: usize = lines
            .iter()
            .flat_map(|l| l.split_inclusive('\n'))
            .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
            .count();
        let max_offset = line_count.saturating_sub(self.viewport_height);
        let new_offset = self.scroll_offset as i32 + delta;
        self.scroll_offset = new_offset.clamp(0, max_offset as i32) as usize;

        if self.scroll_offset == 0 {
            self.frozen_lines = None;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.frozen_lines = None;
    }

    pub fn flush_screen(&mut self) {
        self.parser = vt100::Parser::new(24, 80, 0);
        self.scrollback.clear();
        self.frozen_lines = None;
        self.scroll_offset = 0;
    }

    pub fn poll_serial(&mut self) {
        self.tick_errors();

        if self.connection.is_none() && self.hold && self.screen == Screen::Terminal {
            if let Some(at) = self.reconnect_at {
                if Instant::now() >= at {
                    self.reconnect_at = None;
                    match serial::open(&self.active_port, &self.port_config.clone()) {
                        Ok((tx, rx)) => {
                            self.connection = Some((tx, rx));
                            self.errors.clear();
                        }
                        Err(_) => {
                            self.reconnect_at = Some(Instant::now() + Duration::from_secs(1));
                        }
                    }
                }
            } else {
                self.reconnect_at = Some(Instant::now() + Duration::from_secs(1));
            }
        }

        if let Some((_, rx)) = &self.connection {
            loop {
                match rx.try_recv() {
                    Ok(SerialEvent::Data(bytes)) => {
                        self.parser.process(&bytes);
                        {
                            let stripped = strip_ansi_escapes::strip(&bytes);
                            let text = String::from_utf8_lossy(&stripped);
                            for chunk in text.split_inclusive('\n') {
                                if let Some(last) = self.scrollback.last_mut()
                                    && !last.ends_with('\n')
                                {
                                    last.push_str(chunk);
                                    continue;
                                }
                                self.scrollback.push(chunk.to_string());
                            }
                            if self.scrollback.len() > MAX_SCROLLBACK {
                                self.scrollback
                                    .drain(..self.scrollback.len() - MAX_SCROLLBACK);
                            }
                        }
                    }
                    Ok(SerialEvent::Error(e)) => {
                        self.connection = None;
                        if self.hold {
                            self.reconnect_at = Some(Instant::now() + Duration::from_secs(1));
                        } else {
                            self.push_error(friendly_serial_error(&e));
                            self.screen = Screen::PortSelect;
                        }
                        break;
                    }
                    Ok(SerialEvent::Disconnected) => {
                        self.connection = None;
                        if self.hold {
                            self.reconnect_at = Some(Instant::now() + Duration::from_secs(1));
                        } else {
                            self.screen = Screen::PortSelect;
                        }
                        break;
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

fn port_sort_key(name: &str) -> (&str, u32) {
    let num_start = name
        .rfind(|c: char| !c.is_ascii_digit())
        .map(|i| i + 1)
        .unwrap_or(0);
    let prefix = &name[..num_start];
    let num: u32 = name[num_start..].parse().unwrap_or(0);
    (prefix, num)
}

fn port_priority(name: &str) -> u8 {
    let base = name.rsplit('/').next().unwrap_or(name);
    const COMMON: &[&str] = &["ttyUSB", "ttyACM", "ttyAMA", "COM", "cu."];
    if COMMON.iter().any(|p| base.starts_with(p)) {
        0
    } else {
        1
    }
}

fn sorted_ports(mut ports: Vec<SerialPortInfo>) -> Vec<SerialPortInfo> {
    ports.sort_by(|a, b| {
        let (ap, an) = port_sort_key(&a.port_name);
        let (bp, bn) = port_sort_key(&b.port_name);
        port_priority(&a.port_name)
            .cmp(&port_priority(&b.port_name))
            .then(ap.cmp(bp))
            .then(an.cmp(&bn))
    });
    ports
}

fn friendly_serial_error(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("permission denied") || lower.contains("access denied") {
        "Permission denied - try running with sudo or check port permissions".to_string()
    } else if lower.contains("no such file") || lower.contains("not found") {
        "Port not found - device may have been disconnected".to_string()
    } else if lower.contains("broken pipe") || lower.contains("device disconnected") {
        "Device disconnected unexpectedly".to_string()
    } else if lower.contains("resource busy") || lower.contains("access is denied") {
        "Port is busy - already in use by another application".to_string()
    } else if lower.contains("timed out") {
        "Connection timed out".to_string()
    } else {
        raw.to_string()
    }
}
