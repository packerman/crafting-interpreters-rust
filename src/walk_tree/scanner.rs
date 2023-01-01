use std::str::Chars;

use anyhow::Result;

use super::{error, token::Token, token_kind::TokenKind};

pub struct ScanTokens<'a> {
    chars: Chars<'a>,
    current: String,
    line: usize,
}

impl<'a> ScanTokens<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            current: String::new(),
            line: 1,
        }
    }

    fn scan_token(&mut self) -> Option<Result<Token>> {
        self.advance().map(|c| match c {
            '(' => self.emit_token(TokenKind::LeftParen),
            ')' => self.emit_token(TokenKind::RightParen),
            '{' => self.emit_token(TokenKind::LeftBrace),
            '}' => self.emit_token(TokenKind::RightBrace),
            ',' => self.emit_token(TokenKind::Comma),
            '.' => self.emit_token(TokenKind::Dot),
            '-' => self.emit_token(TokenKind::Minus),
            '+' => self.emit_token(TokenKind::Plus),
            ';' => self.emit_token(TokenKind::Semicolon),
            '*' => self.emit_token(TokenKind::Star),
            _ => error::error(self.line, "Unexpected character"),
        })
    }

    fn advance(&mut self) -> Option<char> {
        let next = self.chars.next();
        if let Some(c) = next {
            self.current.push(c)
        }
        next
    }

    fn emit_token(&mut self, kind: TokenKind) -> Result<Token> {
        let token = Token::new(kind, String::from(&self.current), self.line);
        self.current.clear();
        Ok(token)
    }
}

impl Iterator for ScanTokens<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        error::try_until_ok(|| self.scan_token())
    }
}
