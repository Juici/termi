use anyhow::Result;
use clap::Parser;

mod kitty;

/// Terminal support utility.
#[derive(Parser)]
#[command(version)]
enum Cli {
    /// Query support for progressive keyboard enhancement.
    Kitty,
}

fn main() -> Result<()> {
    match Cli::parse() {
        Cli::Kitty => kitty::main(),
    }
}
