use anyhow::Result;

use super::{error, token_kind::TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize) -> Self {
        Self { kind, lexeme, line }
    }

    pub fn error<T>(&self, message: &str) -> Result<T> {
        if self.kind == TokenKind::Eof {
            error::error_at(self.line, " at end", message)
        } else {
            error::error_at(self.line, &format!(" at '{}'", self.lexeme), message)
        }
    }
}
