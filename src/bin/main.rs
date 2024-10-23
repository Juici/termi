use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

mod desktop_notifications;
mod keyboard_enhancement;

/// Terminal support utility.
#[derive(Parser)]
#[command(version)]
enum Cli {
    /// Query support for progressive keyboard enhancement.
    KeyboardEnhancement,
    /// Query support for desktop notifications.
    DesktopNotifications,
}

pub fn main() -> Result<ExitCode> {
    match Cli::parse() {
        Cli::KeyboardEnhancement => keyboard_enhancement::main(),
        Cli::DesktopNotifications => desktop_notifications::main(),
    }
}
