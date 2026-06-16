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

impl From<DataBitsArg> for DataBits {
    fn from(value: DataBitsArg) -> Self {
        match value {
            DataBitsArg::Five => Self::Five,
            DataBitsArg::Six => Self::Six,
            DataBitsArg::Seven => Self::Seven,
            DataBitsArg::Eight => Self::Eight,
        }
    }
}

impl From<StopBitsArg> for StopBits {
    fn from(value: StopBitsArg) -> Self {
        match value {
            StopBitsArg::One => Self::One,
            StopBitsArg::Two => Self::Two,
        }
    }
}

impl From<ParityArg> for Parity {
    fn from(value: ParityArg) -> Self {
        match value {
            ParityArg::None => Self::None,
            ParityArg::Even => Self::Even,
            ParityArg::Odd => Self::Odd,
        }
    }
}

impl From<FlowControlArg> for FlowControl {
    fn from(value: FlowControlArg) -> Self {
        match value {
            FlowControlArg::None => Self::None,
            FlowControlArg::Software => Self::Software,
            FlowControlArg::Hardware => Self::Hardware,
        }
    }
}

/// cli arg > file config > default
fn resolve<A: Into<T>, F, T>(cli: Option<A>, file: Option<F>, parse: fn(F) -> T, default: T) -> T {
    cli.map(Into::into)
        .or_else(|| file.map(parse))
        .unwrap_or(default)
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

    /// Write a default config file
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

    let file = match cfg::load() {
        Ok(file) => file,
        Err(error) => {
            eprintln!(
                "error: failed to load config file ({}): {error}",
                cfg::config_path_display()
            );
            return None;
        }
    };

    let errors = cfg::validate(&file);
    if !errors.is_empty() {
        eprintln!(
            "error: invalid values in config file ({})",
            cfg::config_path_display()
        );
        for error in &errors {
            eprintln!("  {error}");
        }
        return None;
    }

    let hold = if cli.no_keep_open {
        false
    } else if cli.keep_open {
        true
    } else {
        file.behavior.keep_open.unwrap_or(true)
    };

    let local_echo = cli.local_echo || file.behavior.local_echo.unwrap_or(false);

    Some(Args {
        port: cli.port,
        config: PortConfig {
            baud: cli.baud.or(file.serial.baud).unwrap_or(115200),
            data_bits: resolve(
                cli.data_bits,
                file.serial.data_bits,
                cfg::parse_data_bits,
                DataBits::Eight,
            ),
            stop_bits: resolve(
                cli.stop_bits,
                file.serial.stop_bits,
                cfg::parse_stop_bits,
                StopBits::One,
            ),
            parity: resolve(
                cli.parity,
                file.serial.parity.as_deref(),
                cfg::parse_parity,
                Parity::None,
            ),
            flow_control: resolve(
                cli.flow_control,
                file.serial.flow_control.as_deref(),
                cfg::parse_flow_control,
                FlowControl::None,
            ),
            #[cfg(unix)]
            no_lock: cli.no_lock,
        },
        hold,
        local_echo,
        outgoing_newline: resolve(
            cli.outgoing_newline,
            file.behavior.outgoing_newline.as_deref(),
            cfg::parse_newline,
            NewlineEncoding::CR,
        ),
    })
}
