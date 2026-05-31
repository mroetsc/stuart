use serialport::SerialPortInfo;
use std::sync::mpsc::{Receiver, Sender};

use crate::serial::{self, Command, SerialEvent};

#[derive(Debug)]
pub enum Screen {
    PortSelect,
    Terminal,
}

pub struct App {
    pub screen: Screen,
    pub ports: Vec<SerialPortInfo>,
    pub selected: usize,
    pub exit: bool,
    pub connection: Option<(Sender<Command>, Receiver<SerialEvent>)>,
    pub received: Vec<String>,
    pub error: Option<String>,
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
            received: Vec::new(),
            error: None,
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
            received: Vec::new(),
            error,
        }
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
                    self.received.clear();
                    self.connection = Some((tx, rx));
                    self.screen = Screen::Terminal;
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

    pub fn poll_serial(&mut self) {
        if let Some((_, rx)) = &self.connection {
            loop {
                match rx.try_recv() {
                    Ok(SerialEvent::Data(bytes)) => {
                        self.received
                            .push(String::from_utf8_lossy(&bytes).into_owned());
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
