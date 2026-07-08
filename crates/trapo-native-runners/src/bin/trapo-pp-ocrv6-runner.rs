//! Process-contract wrapper for the native Trapo PP-OCRv6 engine.

use std::{
    io::{self, Write},
    process::ExitCode,
};

#[path = "../ppocrv6/mod.rs"]
mod ppocrv6;

fn main() -> ExitCode {
    match ppocrv6::run() {
        Ok(text) => {
            let _ = io::stdout().write_all(text.as_bytes());
            ExitCode::SUCCESS
        }
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            ExitCode::from(1)
        }
    }
}
