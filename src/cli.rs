use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{Shell, generate};

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

#[derive(Parser)]
#[command(
    version,
    about,
    // a bit hacky but works
    after_help = "\x1b[1m\x1b[4mExtra:\x1b[0m\n  --completions <SHELL>  Generate shell completions [possible values: bash, elvish, fish, powershell, zsh]"
)]
struct Cli {
    #[arg(help = "The port to open")]
    port: Option<String>,

    #[arg(
        short,
        long,
        value_name = "BAUDRATE",
        default_value = "115200",
        help = "Baud rate"
    )]
    baud: u32,

    #[arg(
        short,
        long,
        default_value = "8",
        value_name = "BITS",
        help = "Data bits"
    )]
    data_bits: DataBitsArg,

    #[arg(
        short,
        long,
        default_value = "1",
        value_name = "BITS",
        help = "Stop bits"
    )]
    stop_bits: StopBitsArg,

    #[arg(
        short,
        long,
        default_value = "none",
        value_name = "PARITY",
        help = "Parity"
    )]
    parity: ParityArg,

    #[arg(
        short,
        long,
        default_value = "none",
        value_name = "FLOW",
        help = "Flow control"
    )]
    flow_control: FlowControlArg,

    #[arg(
        short,
        long = "keep-open",
        default_value = "true",
        overrides_with = "no_keep_open",
        help = "Keep terminal open and try to reconnect if the device disconnects [default]"
    )]
    keep_open: bool,

    #[arg(
        long = "no-keep-open",
        overrides_with = "keep_open",
        help = "Exit to port select when device disconnects"
    )]
    no_keep_open: bool,

    #[arg(
        long,
        value_name = "SHELL",
        help = "Generate shell completions",
        hide = true
    )]
    completions: Option<Shell>,
}

pub struct Args {
    pub port: Option<String>,
    pub config: PortConfig,
    pub hold: bool,
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
        },
        hold: !cli.no_keep_open,
    })
}
