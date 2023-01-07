use std::{fs, process::ExitCode};

use anyhow::{Context, Result};
use rustyline::{error::ReadlineError, Editor};

use crate::walk_tree::exit_code;

use super::{error::ErrorReporter, interpreter::Interpreter, parser::Parser, scanner::Scanner};

pub struct Lox<'a> {
    scanner: Scanner<'a>,
    interpreter: Interpreter<'a>,
    error_reporter: &'a ErrorReporter,
}

impl<'a> Lox<'a> {
    pub fn new(error_reporter: &'a ErrorReporter) -> Self {
        Self {
            scanner: Scanner::new(error_reporter),
            interpreter: Interpreter::new(error_reporter),
            error_reporter,
        }
    }

    pub fn run_file(&self, path: &str) -> Result<ExitCode> {
        let source = fs::read_to_string(path)?;
        self.run(source);
        Ok(if self.error_reporter.had_error() {
            exit_code::data_err()
        } else if self.error_reporter.had_runtime_error() {
            exit_code::software()
        } else {
            ExitCode::SUCCESS
        })
    }

    pub fn run_prompt(&self) -> Result<ExitCode> {
        let mut editor = Editor::<()>::new()?;
        loop {
            let readline = editor.readline("> ");
            match readline {
                Ok(line) => {
                    editor.add_history_entry(line.as_str());
                    self.run(line);
                    self.error_reporter.reset();
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
                    eprintln!("Readline error: {:?}", err);
                    break;
                }
            }
        }
        editor.save_history("lox_history.txt")?;
        Ok(ExitCode::SUCCESS)
    }

    fn run(&self, source: String) -> Result<()> {
        let tokens: Vec<_> = self.scanner.scan_tokens(&source).collect();
        let mut parser = Parser::new(tokens, self.error_reporter);
        let statements = parser.parse().context("Failed to parse")?;
        if self.error_reporter.had_error() {
            return Ok(());
        }
        self.interpreter
            .interpret(&statements)
            .context("Failed to run")
    }
}
