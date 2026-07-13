use ratatui::layout::Rect;
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

pub struct TerminalView {
    pub parser: vt100::Parser,
    pub scrollback: Vec<String>,
    pub frozen_lines: Option<Vec<String>>,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub output_rect: Rect,
    pub visible_lines: Vec<String>,
}

impl Default for TerminalView {
    fn default() -> Self {
        Self {
            parser: vt100::Parser::new(24, 80, 0),
            scrollback: Vec::new(),
            frozen_lines: None,
            scroll_offset: 0,
            viewport_height: 24,
            output_rect: Rect::default(),
            visible_lines: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct SelectionState {
    pub anchor: Option<(usize, usize)>,
    pub current: Option<(usize, usize)>,
    pub active: bool,
}

impl SelectionState {
    pub fn start(&mut self, row: usize, col: usize) {
        self.anchor = Some((row, col));
        self.current = Some((row, col));
        self.active = true;
    }

    pub fn update(&mut self, row: usize, col: usize) {
        if self.active {
            self.current = Some((row, col));
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.current = None;
        self.active = false;
    }

    pub fn range(&self) -> Option<((usize, usize), (usize, usize))> {
        let a = self.anchor?;
        let b = self.current?;
        Some(if a <= b { (a, b) } else { (b, a) })
    }
}

#[derive(Default)]
pub struct SettingsUi {
    pub show: bool,
    pub cursor: usize,
    pub baud_input: Option<String>,
}

#[derive(Default)]
pub struct LineInput {
    pub buffer: String,
    pub history: Vec<String>,
    pub history_pos: Option<usize>,
    saved: String,
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
    pub hold: bool,
    pub paused: bool,
    pub reconnect_at: Option<Instant>,
    pub keyboard_enhanced: bool,
    pub local_echo: bool,
    pub input_mode: InputMode,
    pub outgoing_newline: NewlineEncoding,
    pub incoming_newline: NewlineEncoding,
    pub view: TerminalView,
    pub selection: SelectionState,
    pub settings: SettingsUi,
    pub line: LineInput,
    clipboard: Option<arboard::Clipboard>,
}

const MAX_SCROLLBACK: usize = 10000;

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::PortSelect,
            ports: Vec::new(),
            selected: 0,
            exit: false,
            connection: None,
            errors: Vec::new(),
            terminal_mode: TerminalMode::Insert,
            active_port: String::new(),
            port_config: PortConfig::default(),
            hold: true,
            paused: false,
            reconnect_at: None,
            keyboard_enhanced: false,
            local_echo: false,
            input_mode: InputMode::Direct,
            outgoing_newline: NewlineEncoding::CR,
            incoming_newline: NewlineEncoding::CRLF,
            view: TerminalView::default(),
            selection: SelectionState::default(),
            settings: SettingsUi::default(),
            line: LineInput::default(),
            clipboard: arboard::Clipboard::new().ok(),
        }
    }
}

impl App {
    pub fn new(hold: bool, keyboard_enhanced: bool) -> Self {
        Self {
            ports: sorted_ports(serialport::available_ports().unwrap_or_default()),
            hold,
            keyboard_enhanced,
            ..Default::default()
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
            connection,
            errors,
            active_port: port_name.to_string(),
            port_config: config,
            hold,
            keyboard_enhanced,
            ..Default::default()
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
        self.view.parser.screen_mut().set_size(rows, cols);
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
                    self.view = TerminalView::default();
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
        self.paused = false;
        self.screen = Screen::PortSelect;
    }

    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.paused = false;
            match serial::open(&self.active_port, &self.port_config.clone()) {
                Ok((tx, rx)) => {
                    self.connection = Some((tx, rx));
                    self.errors.clear();
                }
                Err(e) => {
                    self.push_error(friendly_serial_error(&e.to_string()));
                }
            }
        } else {
            if let Some((tx, _)) = &self.connection {
                let _ = tx.send(Command::Disconnect);
            }
            self.connection = None;
            self.reconnect_at = None;
            self.paused = true;
        }
    }

    pub fn send_bytes(&mut self, bytes: Vec<u8>) {
        if self.paused {
            return;
        }
        if let Some((tx, _)) = &self.connection {
            let _ = tx.send(Command::Write(bytes));
        }
    }

    pub fn send_line(&mut self) {
        let text = std::mem::take(&mut self.line.buffer);
        self.line.history_pos = None;
        self.line.saved.clear();

        if !text.is_empty() && self.line.history.last().map(String::as_str) != Some(&text) {
            self.line.history.push(text.clone());
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
        if self.line.history.is_empty() {
            return;
        }
        match self.line.history_pos {
            None => {
                self.line.saved = self.line.buffer.clone();
                self.line.history_pos = Some(0);
            }
            Some(pos) if pos + 1 < self.line.history.len() => {
                self.line.history_pos = Some(pos + 1);
            }
            _ => return,
        }
        let idx = self.line.history.len() - 1 - self.line.history_pos.unwrap();
        self.line.buffer = self.line.history[idx].clone();
    }

    pub fn history_next(&mut self) {
        match self.line.history_pos {
            None => {}
            Some(0) => {
                self.line.history_pos = None;
                self.line.buffer = std::mem::take(&mut self.line.saved);
            }
            Some(pos) => {
                self.line.history_pos = Some(pos - 1);
                let idx = self.line.history.len() - pos;
                self.line.buffer = self.line.history[idx].clone();
            }
        }
    }

    pub fn echo_local(&mut self, bytes: &[u8]) {
        if self.paused {
            return;
        }
        use crossterm::style::{Color, ResetColor, SetForegroundColor};

        const ECHO_COLOR: Color = Color::Rgb { r: 255, g: 165, b: 0 };

        let mut parser_feed: Vec<u8> = Vec::new();

        // bit hacky but works
        match bytes {
            [0x0d] => {
                // enter advances line
                parser_feed.extend_from_slice(b"\r\n");
                if let Some(last) = self.view.scrollback.last_mut() && !last.ends_with('\n') {
                    last.push('\n');
                }
            }
            [b @ 0x20..=0x7e] => {
                // display everything ascii in color
                let seq = format!("{}{}{}", SetForegroundColor(ECHO_COLOR), *b as char, ResetColor);
                parser_feed.extend_from_slice(seq.as_bytes());
                let ch = *b as char;
                match self.view.scrollback.last_mut() {
                    Some(last) if !last.ends_with('\n') => last.push(ch),
                    _ => self.view.scrollback.push(ch.to_string()),
                }
            }
            _ => {
                // pass anything else like arrow keys through,
                // so parser handles cursor movement
                parser_feed.extend_from_slice(bytes);
            }
        }

        if !parser_feed.is_empty() {
            self.view.parser.process(&parser_feed);
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

    pub fn finish_selection(&mut self) {
        self.selection.active = false;
        if self.selection.anchor == self.selection.current {
            self.selection.clear();
            return;
        }
        let text = self.selected_text();
        if !text.is_empty() && let Some(clipboard) = &mut self.clipboard {
            let _ = clipboard.set_text(text);
        }
    }

    fn selected_text(&self) -> String {
        let Some((start, end)) = self.selection.range() else {
            return String::new();
        };
        let mut lines = Vec::new();
        for row in start.0..=end.0 {
            let Some(line) = self.view.visible_lines.get(row) else {
                continue;
            };
            let chars: Vec<char> = line.chars().collect();
            let from = if row == start.0 { start.1 } else { 0 };
            let to = if row == end.0 {
                (end.1 + 1).min(chars.len())
            } else {
                chars.len()
            };
            lines.push(if from < to {
                chars[from..to].iter().collect::<String>()
            } else {
                String::new()
            });
        }
        lines
            .iter()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn copy_to_clipboard(&mut self) {
        let lines: Vec<String> = self
            .view
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
        self.selection.clear();
        let entering_scroll = self.view.scroll_offset == 0 && delta > 0;
        if entering_scroll {
            self.view.frozen_lines = Some(self.view.scrollback.clone());
        }

        let lines = self.view.frozen_lines.as_ref().unwrap_or(&self.view.scrollback);
        let line_count: usize = lines
            .iter()
            .flat_map(|l| l.split_inclusive('\n'))
            .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
            .count();
        let max_offset = line_count.saturating_sub(self.view.viewport_height);
        let new_offset = self.view.scroll_offset as i32 + delta;
        self.view.scroll_offset = new_offset.clamp(0, max_offset as i32) as usize;

        if self.view.scroll_offset == 0 {
            self.view.frozen_lines = None;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.view.scroll_offset = 0;
        self.view.frozen_lines = None;
    }

    pub fn flush_screen(&mut self) {
        self.view = TerminalView::default();
    }

    pub fn poll_serial(&mut self) {
        self.tick_errors();

        if self.connection.is_none() && self.hold && !self.paused && self.screen == Screen::Terminal {
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
                        if !self.selection.active {
                            self.selection.clear();
                        }
                        self.view.parser.process(&normalize_newlines_for_parser(&bytes, self.incoming_newline));
                        {
                            let stripped = strip_ansi_escapes::strip(&bytes);
                            let text = String::from_utf8_lossy(&stripped);
                            let normalized: std::borrow::Cow<str> = match self.incoming_newline {
                                NewlineEncoding::CR => {
                                    std::borrow::Cow::Owned(text.replace('\r', "\n"))
                                }
                                NewlineEncoding::LF => text,
                                NewlineEncoding::CRLF => {
                                    std::borrow::Cow::Owned(text.replace("\r\n", "\n"))
                                }
                            };
                            for chunk in normalized.split_inclusive('\n') {
                                if let Some(last) = self.view.scrollback.last_mut()
                                    && !last.ends_with('\n')
                                {
                                    last.push_str(chunk);
                                    continue;
                                }
                                self.view.scrollback.push(chunk.to_string());
                            }
                            if self.view.scrollback.len() > MAX_SCROLLBACK {
                                self.view.scrollback
                                    .drain(..self.view.scrollback.len() - MAX_SCROLLBACK);
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

fn normalize_newlines_for_parser(bytes: &[u8], newline: NewlineEncoding) -> Vec<u8> {
    match newline {
        NewlineEncoding::CRLF => bytes.to_vec(),
        NewlineEncoding::CR => {
            let mut out = Vec::with_capacity(bytes.len() * 2);
            for &b in bytes {
                if b == b'\r' {
                    out.push(b'\r');
                    out.push(b'\n');
                } else {
                    out.push(b);
                }
            }
            out
        }
        NewlineEncoding::LF => {
            let mut out = Vec::with_capacity(bytes.len() * 2);
            for &b in bytes {
                if b == b'\n' {
                    out.push(b'\r');
                    out.push(b'\n');
                } else {
                    out.push(b);
                }
            }
            out
        }
    }
}
