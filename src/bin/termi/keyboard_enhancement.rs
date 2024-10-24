use std::process::ExitCode;

use anyhow::Result;

pub fn main() -> Result<ExitCode> {
    let _flags = match termi::feature::keyboard_enhancement::query()? {
        Some(flags) => flags,
        None => return Ok(ExitCode::FAILURE),
    };

    Ok(ExitCode::SUCCESS)
}
