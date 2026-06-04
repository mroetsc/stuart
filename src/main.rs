use std::io::stdout;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::supports_keyboard_enhancement,
};

mod serial;
mod state;
mod ui;

use state::App;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, value_name = "BAUDRATE", help = "Set the baud rate")]
    baud: Option<u32>,

    #[arg(help = "The port to open")]
    port: Option<String>,

    #[arg(
        long,
        help = "Hold the terminal open and reconnect if the device disconnects"
    )]
    hold: bool,

    #[arg(
        long,
        value_name = "SHELL",
        help = "Generate shell completions",
        hide = true
    )]
    completions: Option<Shell>,
}

fn main() {
    let args = Cli::parse();

    if let Some(shell) = args.completions {
        generate(shell, &mut Cli::command(), "stuart", &mut std::io::stdout());
        return;
    }

    let enhanced = matches!(supports_keyboard_enhancement(), Ok(true));

    let mut terminal = ratatui::init();

    if enhanced {
        execute!(
            stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )
        .unwrap();
    }

    let mut app = match args.port {
        Some(port) => App::with_port(&port, args.baud.unwrap_or(115200), args.hold),
        None => App::new(args.hold),
    };

    ui::run(&mut app, &mut terminal).unwrap();

    if enhanced {
        execute!(stdout(), PopKeyboardEnhancementFlags).unwrap();
    }

    ratatui::restore();
}
