use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, value_name = "BAUDRATE", help = "Set the Baudrate")]
    baud: Option<u32>,

    #[arg(help = "The port to be used")]
    port: Option<String>,
}

fn main() {
    let _args = Cli::parse();
}
