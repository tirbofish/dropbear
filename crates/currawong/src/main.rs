mod cli;
mod commands;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    if let Err(e) = cli.match_command() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
