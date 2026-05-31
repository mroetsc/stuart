use serialport::SerialPortInfo;

#[derive(Debug)]
pub enum Screen {
    PortSelect,
    Terminal,
}

#[derive(Debug)]
pub struct App {
    pub screen: Screen,
    pub ports: Vec<SerialPortInfo>,
    pub selected: usize,
    pub exit: bool,
}

impl App {
    pub fn new() -> Self {
        let ports = serialport::available_ports().unwrap_or_default();
        Self {
            screen: Screen::PortSelect,
            ports,
            selected: 0,
            exit: false,
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.ports.is_empty() {
            return;
        }
        let len = self.ports.len() as i32;
        self.selected = ((self.selected as i32 + delta).rem_euclid(len)) as usize;
    }
}
