use std::io::stdout;

use clap::Parser;
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
}

fn main() {
    let args = Cli::parse();

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
        Some(port) => App::with_port(&port, args.baud.unwrap_or(115200)),
        None => App::new(),
    };

    ui::run(&mut app, &mut terminal).unwrap();

    if enhanced {
        execute!(stdout(), PopKeyboardEnhancementFlags).unwrap();
    }

    ratatui::restore();
}
