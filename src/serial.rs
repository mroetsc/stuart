use std::io::ErrorKind;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

pub use serialport::{DataBits, FlowControl, Parity, StopBits};

pub const BAUD_RATES: &[u32] = &[9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];

pub enum Command {
    Write(Vec<u8>),
    Disconnect,
}

pub enum SerialEvent {
    Data(Vec<u8>),
    Error(String),
    Disconnected,
}

#[derive(Clone)]
pub struct PortConfig {
    pub baud: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub no_lock: bool,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            baud: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            no_lock: false,
        }
    }
}

pub fn open(
    port_name: &str,
    config: &PortConfig,
) -> Result<(Sender<Command>, Receiver<SerialEvent>), serialport::Error> {
    let port = serialport::new(port_name, config.baud)
        .data_bits(config.data_bits)
        .stop_bits(config.stop_bits)
        .parity(config.parity)
        .flow_control(config.flow_control)
        .exclusive(!config.no_lock)
        .timeout(Duration::from_millis(10))
        .open()?;

    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (event_tx, event_rx) = mpsc::channel::<SerialEvent>();

    thread::spawn(move || run(port, cmd_rx, event_tx));

    Ok((cmd_tx, event_rx))
}

fn run(
    mut port: Box<dyn serialport::SerialPort>,
    cmd_rx: Receiver<Command>,
    event_tx: Sender<SerialEvent>,
) {
    let mut buf = vec![0u8; 256];

    loop {
        match cmd_rx.try_recv() {
            Ok(Command::Write(bytes)) => {
                if let Err(e) = port.write_all(&bytes) {
                    let _ = event_tx.send(SerialEvent::Error(e.to_string()));
                }
            }
            Ok(Command::Disconnect) => {
                let _ = event_tx.send(SerialEvent::Disconnected);
                break;
            }
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }

        match port.read(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                let _ = event_tx.send(SerialEvent::Data(buf[..n].to_vec()));
            }
            Err(e) if e.kind() == ErrorKind::TimedOut => {}
            Err(e) => {
                let _ = event_tx.send(SerialEvent::Error(e.to_string()));
                break;
            }
        }
    }
}
