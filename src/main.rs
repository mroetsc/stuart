use std::io::stdout;

use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::supports_keyboard_enhancement,
};

mod cli;
mod serial;
mod state;
mod ui;

use state::App;

fn main() {
    let Some(args) = cli::parse() else { return };

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
        Some(port) => App::with_port(&port, args.config, args.hold, enhanced),
        None => {
            let mut app = App::new(args.hold, enhanced);
            app.port_config = args.config;
            app
        }
    };
    app.local_echo = args.local_echo;

    ui::run(&mut app, &mut terminal).unwrap();

    if enhanced {
        execute!(stdout(), PopKeyboardEnhancementFlags).unwrap();
    }

    ratatui::restore();
}
