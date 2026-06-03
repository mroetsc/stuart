use serialport::SerialPortInfo;
use std::sync::mpsc::{Receiver, Sender};

use crate::serial::{self, Command, SerialEvent};

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

pub struct App {
    pub screen: Screen,
    pub ports: Vec<SerialPortInfo>,
    pub selected: usize,
    pub exit: bool,
    pub connection: Option<(Sender<Command>, Receiver<SerialEvent>)>,
    pub error: Option<String>,
    pub terminal_mode: TerminalMode,
    pub active_port: String,
    pub current_baud: u32,
    pub parser: vt100::Parser,
}

impl App {
    pub fn new() -> Self {
        let ports = serialport::available_ports().unwrap_or_default();
        Self {
            screen: Screen::PortSelect,
            ports,
            selected: 0,
            exit: false,
            connection: None,
            error: None,
            terminal_mode: TerminalMode::Insert,
            active_port: String::new(),
            current_baud: 0,
            parser: vt100::Parser::new(24, 80, 0),
        }
    }

    pub fn with_port(port_name: &str, baud: u32) -> Self {
        let (connection, error, screen) = match serial::open(port_name, baud) {
            Ok((tx, rx)) => (Some((tx, rx)), None, Screen::Terminal),
            Err(e) => (None, Some(e.to_string()), Screen::PortSelect),
        };
        Self {
            screen,
            ports: serialport::available_ports().unwrap_or_default(),
            selected: 0,
            exit: false,
            connection,
            error,
            terminal_mode: TerminalMode::Insert,
            active_port: port_name.to_string(),
            current_baud: baud,
            parser: vt100::Parser::new(24, 80, 0),
        }
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
            match serial::open(&port.port_name, 115200) {
                Ok((tx, rx)) => {
                    self.error = None;
                    self.parser = vt100::Parser::new(24, 80, 0);
                    self.active_port = port.port_name.clone();
                    self.current_baud = 115200;
                    self.connection = Some((tx, rx));
                    self.screen = Screen::Terminal;
                    self.terminal_mode = TerminalMode::Insert;
                }
                Err(e) => {
                    self.error = Some(e.to_string());
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
            .position(|&r| r == self.current_baud)
            .unwrap_or(4);
        let next = (current as i32 + delta).clamp(0, RATES.len() as i32 - 1) as usize;
        let new_baud = RATES[next];
        if new_baud == self.current_baud {
            return;
        }
        self.connection = None;
        std::thread::sleep(std::time::Duration::from_millis(100));
        match serial::open(&self.active_port, new_baud) {
            Ok((tx, rx)) => {
                self.current_baud = new_baud;
                self.connection = Some((tx, rx));
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.screen = Screen::PortSelect;
            }
        }
    }

    pub fn flush_screen(&mut self) {
        self.parser = vt100::Parser::new(24, 80, 0);
    }

    pub fn poll_serial(&mut self) {
        if let Some((_, rx)) = &self.connection {
            loop {
                match rx.try_recv() {
                    Ok(SerialEvent::Data(bytes)) => {
                        self.parser.process(&bytes);
                    }
                    Ok(SerialEvent::Error(e)) => {
                        self.error = Some(e);
                        self.connection = None;
                        self.screen = Screen::PortSelect;
                        break;
                    }
                    Ok(SerialEvent::Disconnected) => {
                        self.connection = None;
                        self.screen = Screen::PortSelect;
                        break;
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
