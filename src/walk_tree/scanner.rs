use std::{iter::Peekable, str::Chars};

use anyhow::Result;

use super::{error, token::Token, token_kind::TokenKind};

pub struct ScanTokens<'a> {
    chars: Peekable<Chars<'a>>,
    current: String,
    line: usize,
    is_at_end: bool,
}

impl<'a> ScanTokens<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            current: String::new(),
            line: 1,
            is_at_end: false,
        }
    }

    fn scan_token(&mut self) -> Option<Result<Token>> {
        if let Some(c) = self.advance() {
            match c {
                '(' => Some(self.emit_token(TokenKind::LeftParen)),
                ')' => Some(self.emit_token(TokenKind::RightParen)),
                '{' => Some(self.emit_token(TokenKind::LeftBrace)),
                '}' => Some(self.emit_token(TokenKind::RightBrace)),
                ',' => Some(self.emit_token(TokenKind::Comma)),
                '.' => Some(self.emit_token(TokenKind::Dot)),
                '-' => Some(self.emit_token(TokenKind::Minus)),
                '+' => Some(self.emit_token(TokenKind::Plus)),
                ';' => Some(self.emit_token(TokenKind::Semicolon)),
                '*' => Some(self.emit_token(TokenKind::Star)),
                '!' => {
                    let token_kind = if self.match_char('=') {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    };
                    Some(self.emit_token(token_kind))
                }
                '=' => {
                    let token_kind = if self.match_char('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    Some(self.emit_token(token_kind))
                }
                '<' => {
                    let token_kind = if self.match_char('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    Some(self.emit_token(token_kind))
                }
                '>' => {
                    let token_kind = if self.match_char('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    Some(self.emit_token(token_kind))
                }
                '/' => {
                    if self.match_char('/') {
                        while self.chars.peek().map_or(false, |c| *c != '\n') {
                            self.chars.next();
                        }
                        None
                    } else {
                        Some(self.emit_token(TokenKind::Slash))
                    }
                }
                ' ' => None,
                '\r' => None,
                '\t' => None,
                '\n' => {
                    self.line += 1;
                    None
                }
                _ => Some(error::error(self.line, "Unexpected character")),
            }
        } else {
            Some(self.emit_token(TokenKind::Eof))
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.chars.peek() {
            if *c != expected {
                false
            } else {
                self.chars.next();
                true
            }
        } else {
            false
        }
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
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_end {
            return None;
        }
        loop {
            if let Some(token) = self.scan_token() {
                if token.as_ref().map_or(false, |token| token.is_eof()) {
                    self.is_at_end = true;
                }
                return Some(token);
            }
        }
    }
}
