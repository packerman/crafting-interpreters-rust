use std::{fs, io::Write, process::ExitCode};

use anyhow::Result;
use rustyline::{error::ReadlineError, Editor};

use crate::walk_tree::exit_code;

use super::{error::ErrorReporter, interpreter::Interpreter, parser::Parser, scanner::Scanner};

pub struct Lox<'a, W> {
    scanner: Scanner<'a>,
    interpreter: Interpreter<'a, W>,
    error_reporter: &'a ErrorReporter,
}

impl<'a, W> Lox<'a, W>
where
    W: Write + 'a,
{
    pub fn new(error_reporter: &'a ErrorReporter, output: W) -> Self {
        Self {
            scanner: Scanner::new(error_reporter),
            interpreter: Interpreter::new_with_output(error_reporter, output),
            error_reporter,
        }
    }

    pub fn run_file(&mut self, path: &str) -> Result<ExitCode> {
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

    pub fn run_prompt(&mut self) -> Result<ExitCode> {
        let mut editor = Editor::<()>::new()?;
        loop {
            let read_line = editor.readline("> ");
            match read_line {
                Ok(line) => {
                    editor.add_history_entry(line.as_str());
                    self.run_interactively(line);
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
                    eprintln!("Read line error: {:?}", err);
                    break;
                }
            }
        }
        editor.save_history("lox_history.txt")?;
        Ok(ExitCode::SUCCESS)
    }

    fn run(&mut self, source: String) {
        let tokens: Vec<_> = self.scanner.scan_tokens(&source).collect();
        let mut parser = Parser::new(tokens, self.error_reporter);
        let statements = parser.parse().unwrap_or_default();
        if self.error_reporter.had_error() {
            return;
        }
        self.interpreter.interpret(&statements)
    }

    fn run_interactively(&mut self, line: String) {
        let result = self
            .error_reporter
            .run_without_printing_error(|| self.try_evaluate_expression(&line));
        if result.is_err() {
            self.error_reporter.reset();
            return;
        }
        if self.error_reporter.had_error() {
            self.error_reporter.reset();
            self.run(line);
        }
        self.error_reporter.reset();
    }

    fn try_evaluate_expression(&mut self, source: &str) -> Result<()> {
        let tokens: Vec<_> = self.scanner.scan_tokens(source).collect();
        let mut parser = Parser::new(tokens, self.error_reporter);
        if let Some(expression) = parser.expression() {
            self.interpreter.evaluate_and_print(&expression)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn is_able_to_enter_expressions_1() {
        assert_prints(vec!["print 2+2;".into(), "2+2".into()], b"4\n4\n")
    }

    #[test]
    fn is_able_to_enter_expressions_2() {
        assert_prints(vec!["var a = 1;".into(), "a".into()], b"1\n")
    }

    #[test]
    fn is_able_to_enter_expressions_3() {
        assert_prints(
            vec![
                "var a = 1;".into(),
                "var b = 2;".into(),
                "2-1".into(),
                "b-a".into(),
            ],
            b"1\n1\n",
        )
    }

    fn assert_prints(source: Vec<String>, value: &[u8]) {
        assert_eq!(test_interpreter_output(source).unwrap(), value)
    }

    fn test_interpreter_output(lines: Vec<String>) -> Result<Vec<u8>> {
        let error_reporter = ErrorReporter::new();
        let mut output = Vec::new();
        let mut lox = Lox::new(&error_reporter, &mut output);
        for line in lines.into_iter() {
            lox.run_interactively(line);
        }
        Ok(output)
    }
}
