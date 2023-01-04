use std::{env, process::ExitCode};

use anyhow::Result;
use crafting_interpreters_rust::walk_tree::{error::ErrorReporter, exit_code, lox::Lox};

fn main() -> Result<ExitCode> {
    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: walk_tree [script]");
        return Ok(exit_code::usage());
    }
    let error_reporter = ErrorReporter::new();
    let lox = Lox::new(&error_reporter);
    if args.len() == 2 {
        lox.run_file(&args[1])
    } else {
        lox.run_prompt()
    }
}
