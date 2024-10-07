use std::io::{self, Write};
use std::process;

use anyhow::Result;

pub fn main() -> Result<()> {
    let (output, exit_code) = match crossterm::terminal::supports_keyboard_enhancement()? {
        true => (b"1\n", 0),
        false => (b"0\n", 1),
    };

    let mut stdout = io::stdout().lock();

    stdout.write_all(output)?;
    stdout.flush()?;

    process::exit(exit_code);
}
