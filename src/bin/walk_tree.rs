use std::{env, process::ExitCode};

use anyhow::Result;
use crafting_interpreters_rust::walk_tree::{exit_code, lox};

fn main() -> Result<ExitCode> {
    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: walk_tree [script]");
        return Ok(exit_code::usage());
    }
    if args.len() == 2 {
        lox::run_file(&args[1])
    } else {
        lox::run_prompt()
    }
}
