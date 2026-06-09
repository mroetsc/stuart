use serialport::SerialPortInfo;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crate::serial::{self, Command, PortConfig, SerialEvent};

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
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub hold: bool,
    pub reconnect_at: Option<Instant>,
    pub show_settings: bool,
    clipboard: Option<arboard::Clipboard>,
}

const MAX_SCROLLBACK: usize = 10000;

impl App {
    pub fn new(hold: bool) -> Self {
        let ports = serialport::available_ports().unwrap_or_default();
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
            scroll_offset: 0,
            viewport_height: 24,
            hold,
            reconnect_at: None,
            show_settings: false,
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    pub fn with_port(port_name: &str, config: PortConfig, hold: bool) -> Self {
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
            ports: serialport::available_ports().unwrap_or_default(),
            selected: 0,
            exit: false,
            connection,
            errors,
            terminal_mode: TerminalMode::Insert,
            active_port: port_name.to_string(),
            port_config: config,
            parser: vt100::Parser::new(24, 80, 0),
            scrollback: Vec::new(),
            scroll_offset: 0,
            viewport_height: 24,
            hold,
            reconnect_at: None,
            show_settings: false,
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    pub fn refresh_ports(&mut self) {
        self.ports = serialport::available_ports().unwrap_or_default();
        self.selected = self.selected.min(self.ports.len().saturating_sub(1));
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

    pub fn change_baud(&mut self, delta: i32) {
        const RATES: &[u32] = &[9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
        let current = RATES
            .iter()
            .position(|&r| r == self.port_config.baud)
            .unwrap_or(4);
        let next = (current as i32 + delta).clamp(0, RATES.len() as i32 - 1) as usize;
        let new_baud = RATES[next];
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
        let line_count: usize = self
            .scrollback
            .iter()
            .flat_map(|l| l.split_inclusive('\n'))
            .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
            .count();
        let max_offset = line_count.saturating_sub(self.viewport_height);
        let new_offset = self.scroll_offset as i32 + delta;
        self.scroll_offset = new_offset.clamp(0, max_offset as i32) as usize;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn flush_screen(&mut self) {
        self.parser = vt100::Parser::new(24, 80, 0);
        self.scrollback.clear();
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
