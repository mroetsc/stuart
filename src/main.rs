use clap::Parser;

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

    match args.port {
        Some(port) => {
            todo!("open port directly");
        }
        None => {
            let mut terminal = ratatui::init();
            let mut app = App::new();
            ui::run(&mut app, &mut terminal).unwrap();
            ratatui::restore();
        }
    }
}
