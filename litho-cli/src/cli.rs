use std::env::args;
use std::process::ExitCode;

use super::commands;

pub fn cli() -> ExitCode {
    let command = args().skip(1).next();

    match command.as_deref() {
        Some("generate") => {}
        Some("--version" | "-v" | "version") => return commands::version(),
        _ => {}
    }

    commands::generate()
}
