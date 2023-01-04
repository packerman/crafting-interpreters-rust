use std::cell::Cell;

use super::{token::Token, token_kind::TokenKind};

pub struct ErrorReporter {
    had_error: Cell<bool>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            had_error: Cell::new(false),
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error.get()
    }

    pub fn error(&self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn reset(&self) {
        self.had_error.set(false);
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
}
