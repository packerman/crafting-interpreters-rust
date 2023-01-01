use std::{fs, process::ExitCode};

use anyhow::Result;
use rustyline::{error::ReadlineError, Editor};

use crate::walk_tree::exit_code;

pub fn run_file(path: &str) -> Result<ExitCode> {
    let source = fs::read_to_string(path)?;
    Ok(if let Err(err) = self::run(source) {
        println!("Error while running file: {}", err);
        exit_code::data_err()
    } else {
        ExitCode::SUCCESS
    })
}

pub fn run_prompt() -> Result<ExitCode> {
    let mut editor = Editor::<()>::new()?;
    loop {
        let readline = editor.readline("> ");
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_str());
                let result = self::run(line);
                if let Err(err) = result {
                    println!("Run error: {}", err);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Readline error: {:?}", err);
                break;
            }
        }
    }
    editor.save_history("lox_history.txt")?;
    Ok(ExitCode::SUCCESS)
}

pub fn run(source: String) -> Result<()> {
    println!("Source: {}", source);
    Ok(())
}
