use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};

use crate::config as cfg;
use crate::serial::{DataBits, FlowControl, NewlineEncoding, Parity, PortConfig, StopBits};

#[derive(Clone, ValueEnum)]
enum DataBitsArg {
    #[value(name = "5")]
    Five,
    #[value(name = "6")]
    Six,
    #[value(name = "7")]
    Seven,
    #[value(name = "8")]
    Eight,
}

#[derive(Clone, ValueEnum)]
enum StopBitsArg {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
}

#[derive(Clone, ValueEnum)]
enum ParityArg {
    None,
    Even,
    Odd,
}

#[derive(Clone, ValueEnum)]
enum FlowControlArg {
    None,
    Software,
    Hardware,
}

/// A serial terminal TUI
#[derive(Parser)]
#[command(
    version,
    about,
    long_about = None,
    next_display_order = None,
    disable_help_flag = true,
    disable_version_flag = true,
)]
struct Cli {
    /// Serial port to open
    #[arg()]
    port: Option<String>,

    /// Baud rate
    #[arg(
        short,
        long,
        value_name = "BAUDRATE",
        help_heading = "Serial Settings",
        display_order = 1
    )]
    baud: Option<u32>,

    /// Data bits
    #[arg(
        short,
        long,
        value_name = "BITS",
        help_heading = "Serial Settings",
        display_order = 2
    )]
    data_bits: Option<DataBitsArg>,

    /// Stop bits
    #[arg(
        short,
        long,
        value_name = "BITS",
        help_heading = "Serial Settings",
        display_order = 3
    )]
    stop_bits: Option<StopBitsArg>,

    /// Parity
    #[arg(
        short,
        long,
        value_name = "PARITY",
        help_heading = "Serial Settings",
        display_order = 4
    )]
    parity: Option<ParityArg>,

    /// Flow control
    #[arg(
        short,
        long,
        value_name = "FLOW",
        help_heading = "Serial Settings",
        display_order = 5
    )]
    flow_control: Option<FlowControlArg>,

    /// Echo typed characters locally (for devices that don't echo)
    #[arg(
        short = 'e',
        long = "local-echo",
        help_heading = "Behavior",
        display_order = 6
    )]
    local_echo: bool,

    /// Encoding to send to the device when pressing Enter
    #[arg(
        long = "outgoing-newline",
        value_name = "NEWLINE_ENCODING",
        help_heading = "Behavior",
        display_order = 7
    )]
    outgoing_newline: Option<NewlineEncoding>,

    /// Don't lock the port
    #[cfg(unix)]
    #[arg(long = "no-lock", help_heading = "Behavior", display_order = 8)]
    no_lock: bool,

    /// Keep terminal open and reconnect if the device disconnects [default]
    #[arg(
        short = 'k',
        long = "keep-open",
        overrides_with = "no_keep_open",
        help_heading = "Behavior",
        display_order = 9
    )]
    keep_open: bool,

    /// Exit to port select when device disconnects
    #[arg(
        long = "no-keep-open",
        overrides_with = "keep_open",
        help_heading = "Behavior",
        display_order = 10
    )]
    no_keep_open: bool,

    /// Write a default config file; exits after writing
    #[arg(long = "create-config", help_heading = "Extra", display_order = 11)]
    create_config: bool,

    /// Overwrite existing config file (only valid with --create-config)
    #[arg(long = "force", requires = "create_config", hide = true)]
    force: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL", help_heading = "Extra", display_order = 12)]
    completions: Option<Shell>,

    /// Print help
    #[arg(short, long, action = clap::ArgAction::Help, help_heading = "Options", display_order = 13)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'V', long, action = clap::ArgAction::Version, help_heading = "Options", display_order = 14)]
    version: Option<bool>,
}

pub struct Args {
    pub port: Option<String>,
    pub config: PortConfig,
    pub hold: bool,
    pub local_echo: bool,
    pub outgoing_newline: NewlineEncoding,
}

pub fn parse() -> Option<Args> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        generate(shell, &mut Cli::command(), "stuart", &mut std::io::stdout());
        return None;
    }

    if cli.create_config {
        match cfg::write_default(cli.force) {
            Ok(path) => eprintln!("config written to {}", path.display()),
            Err(e) => eprintln!("error: {e}"),
        }
        return None;
    }

    let file = cfg::load();

    let errors = cfg::validate(&file);
    if !errors.is_empty() {
        eprintln!(
            "error: invalid values in config file ({})",
            cfg::config_path_display()
        );
        for e in &errors {
            eprintln!("  {e}");
        }
        return None;
    }

    let baud = cli.baud.or(file.serial.baud).unwrap_or(115200);

    let data_bits = cli
        .data_bits
        .map(|v| match v {
            DataBitsArg::Five => DataBits::Five,
            DataBitsArg::Six => DataBits::Six,
            DataBitsArg::Seven => DataBits::Seven,
            DataBitsArg::Eight => DataBits::Eight,
        })
        .or_else(|| file.serial.data_bits.map(cfg::parse_data_bits))
        .unwrap_or(DataBits::Eight);

    let stop_bits = cli
        .stop_bits
        .map(|v| match v {
            StopBitsArg::One => StopBits::One,
            StopBitsArg::Two => StopBits::Two,
        })
        .or_else(|| file.serial.stop_bits.map(cfg::parse_stop_bits))
        .unwrap_or(StopBits::One);

    let parity = cli
        .parity
        .map(|v| match v {
            ParityArg::None => Parity::None,
            ParityArg::Even => Parity::Even,
            ParityArg::Odd => Parity::Odd,
        })
        .or_else(|| file.serial.parity.as_deref().map(cfg::parse_parity))
        .unwrap_or(Parity::None);

    let flow_control = cli
        .flow_control
        .map(|v| match v {
            FlowControlArg::None => FlowControl::None,
            FlowControlArg::Software => FlowControl::Software,
            FlowControlArg::Hardware => FlowControl::Hardware,
        })
        .or_else(|| {
            file.serial
                .flow_control
                .as_deref()
                .map(cfg::parse_flow_control)
        })
        .unwrap_or(FlowControl::None);

    let local_echo = if cli.local_echo {
        true
    } else {
        file.behavior.local_echo.unwrap_or(false)
    };

    let outgoing_newline = cli
        .outgoing_newline
        .or_else(|| {
            file.behavior
                .outgoing_newline
                .as_deref()
                .map(cfg::parse_newline)
        })
        .unwrap_or(NewlineEncoding::CR);

    let hold = if cli.no_keep_open {
        false
    } else if cli.keep_open {
        true
    } else {
        file.behavior.keep_open.unwrap_or(true)
    };

    Some(Args {
        port: cli.port,
        config: PortConfig {
            baud,
            data_bits,
            stop_bits,
            parity,
            flow_control,
            #[cfg(unix)]
            no_lock: cli.no_lock,
        },
        hold,
        local_echo,
        outgoing_newline,
    })
}
