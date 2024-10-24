use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod desktop_notifications;
mod keyboard_enhancement;

/// Terminal support utility.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Query the terminal for supported features.
    #[command(subcommand)]
    Query(Query),
}

#[derive(Subcommand)]
enum Query {
    /// Query support for progressive keyboard enhancement.
    KeyboardEnhancement,
    /// Query support for desktop notifications.
    DesktopNotifications,
}

pub fn main() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Command::Query(query) => match query {
            Query::KeyboardEnhancement => keyboard_enhancement::main(),
            Query::DesktopNotifications => desktop_notifications::main(),
        },
    }
}
