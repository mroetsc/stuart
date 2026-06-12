use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};

use crate::serial::{DataBits, FlowControl, Parity, PortConfig, StopBits};

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
        default_value = "115200",
        help_heading = "Serial Settings",
        display_order = 1
    )]
    baud: u32,

    /// Data bits
    #[arg(
        short,
        long,
        value_name = "BITS",
        default_value = "8",
        help_heading = "Serial Settings",
        display_order = 2
    )]
    data_bits: DataBitsArg,

    /// Stop bits
    #[arg(
        short,
        long,
        value_name = "BITS",
        default_value = "1",
        help_heading = "Serial Settings",
        display_order = 3
    )]
    stop_bits: StopBitsArg,

    /// Parity
    #[arg(
        short,
        long,
        value_name = "PARITY",
        default_value = "none",
        help_heading = "Serial Settings",
        display_order = 4
    )]
    parity: ParityArg,

    /// Flow control
    #[arg(
        short,
        long,
        value_name = "FLOW",
        default_value = "none",
        help_heading = "Serial Settings",
        display_order = 5
    )]
    flow_control: FlowControlArg,

    /// Echo typed characters locally (for devices that don't echo)
    #[arg(
        short = 'e',
        long = "local-echo",
        help_heading = "Behavior",
        display_order = 6
    )]
    local_echo: bool,

    /// Don't lock the port
    #[arg(long = "no-lock", help_heading = "Behavior", display_order = 7)]
    no_lock: bool,

    /// Keep terminal open and reconnect if the device disconnects [default]
    #[arg(
        short = 'k',
        long = "keep-open",
        default_value = "true",
        overrides_with = "no_keep_open",
        help_heading = "Behavior",
        display_order = 8
    )]
    keep_open: bool,

    /// Exit to port select when device disconnects
    #[arg(
        long = "no-keep-open",
        overrides_with = "keep_open",
        help_heading = "Behavior",
        display_order = 9
    )]
    no_keep_open: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL", help_heading = "Extra", display_order = 10)]
    completions: Option<Shell>,

    /// Print help
    #[arg(short, long, action = clap::ArgAction::Help, help_heading = "Options", display_order = 11)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'V', long, action = clap::ArgAction::Version, help_heading = "Options", display_order = 12)]
    version: Option<bool>,
}

pub struct Args {
    pub port: Option<String>,
    pub config: PortConfig,
    pub hold: bool,
    pub local_echo: bool,
}

pub fn parse() -> Option<Args> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        generate(shell, &mut Cli::command(), "stuart", &mut std::io::stdout());
        return None;
    }

    Some(Args {
        port: cli.port,
        config: PortConfig {
            baud: cli.baud,
            data_bits: match cli.data_bits {
                DataBitsArg::Five => DataBits::Five,
                DataBitsArg::Six => DataBits::Six,
                DataBitsArg::Seven => DataBits::Seven,
                DataBitsArg::Eight => DataBits::Eight,
            },
            stop_bits: match cli.stop_bits {
                StopBitsArg::One => StopBits::One,
                StopBitsArg::Two => StopBits::Two,
            },
            parity: match cli.parity {
                ParityArg::None => Parity::None,
                ParityArg::Even => Parity::Even,
                ParityArg::Odd => Parity::Odd,
            },
            flow_control: match cli.flow_control {
                FlowControlArg::None => FlowControl::None,
                FlowControlArg::Software => FlowControl::Software,
                FlowControlArg::Hardware => FlowControl::Hardware,
            },
            no_lock: cli.no_lock,
        },
        hold: !cli.no_keep_open,
        local_echo: cli.local_echo,
    })
}
