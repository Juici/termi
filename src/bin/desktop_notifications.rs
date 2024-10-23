use std::process::ExitCode;

use anyhow::Result;

pub fn main() -> Result<ExitCode> {
    let _support = match termi::feature::desktop_notifications::query()? {
        Some(support) => support,
        None => return Ok(ExitCode::FAILURE),
    };

    Ok(ExitCode::SUCCESS)
}
