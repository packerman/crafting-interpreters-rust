use std::{cell::Cell, error::Error, fmt::Display};

use super::token::{Token, TokenKind};

pub struct ErrorReporter {
    had_error: Cell<bool>,
    had_runtime_error: Cell<bool>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            had_error: Cell::new(false),
            had_runtime_error: Cell::new(false),
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error.get()
    }

    pub fn had_runtime_error(&self) -> bool {
        self.had_runtime_error.get()
    }

    pub fn error(&self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn reset(&self) {
        self.had_error.set(false);
        self.had_runtime_error.set(false);
    }

    fn report(&self, line: usize, where_part: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, where_part, message);
        self.had_error.set(true)
    }

    pub fn token_error(&self, token: &Token, message: &str) {
        if token.kind == TokenKind::Eof {
            self.report(token.line, " at end", message)
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message)
        }
    }

    pub fn runtime_error(&self, error: &RuntimeError) {
        eprintln!("{}", error);
        self.had_runtime_error.set(true);
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            token: Some(token),
            message: String::from(message),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(token) = &self.token {
            write!(f, "{}\n[line {}]", self.message, token.line)
        } else {
            write!(f, "Runtime error: {}", self.message)
        }
    }
}

impl From<String> for RuntimeError {
    fn from(message: String) -> Self {
        Self {
            token: None,
            message,
        }
    }
}

impl Error for RuntimeError {}
