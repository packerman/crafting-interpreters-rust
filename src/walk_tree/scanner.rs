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
                        while self.peek().map_or(false, |c| *c != '\n') {
                            self.skip();
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
                '"' => self.string(),
                _ => Some(error::error(self.line, "Unexpected character")),
            }
        } else {
            Some(self.emit_token(TokenKind::Eof))
        }
    }

    fn string(&mut self) -> Option<Result<Token>> {
        while self.peek().map_or(false, |c| *c != '"') {
            if self.peek().map_or(false, |c| *c == '\n') {
                self.line += 1;
            }
            self.advance();
        }
        if self.peek().is_none() {
            Some(error::error(self.line, "Unterminated string."))
        } else {
            self.skip();
            self.current.remove(0);
            Some(self.emit_token(TokenKind::String))
        }
    }

    fn start(&mut self) {
        self.current.clear()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if *c != expected {
                false
            } else {
                self.advance();
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

    fn skip(&mut self) {
        self.chars.next();
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn emit_token(&mut self, kind: TokenKind) -> Result<Token> {
        Ok(Token::new(kind, String::from(&self.current), self.line))
    }
}

impl Iterator for ScanTokens<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_end {
            return None;
        }
        loop {
            self.start();
            if let Some(token) = self.scan_token() {
                if token.as_ref().map_or(false, |token| token.is_eof()) {
                    self.is_at_end = true;
                }
                return Some(token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_works() {
        let tokens: Result<Vec<_>> = ScanTokens::new("// this is a comment").collect();
        assert_eq!(
            tokens.unwrap(),
            vec![Token::new(TokenKind::Eof, "".to_string(), 1)]
        )
    }

    #[test]
    fn grouping_stuff_works() {
        let tokens: Result<Vec<_>> = ScanTokens::new("(( )){}").collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::new(TokenKind::LeftParen, "(".to_string(), 1),
                Token::new(TokenKind::LeftParen, "(".to_string(), 1),
                Token::new(TokenKind::RightParen, ")".to_string(), 1),
                Token::new(TokenKind::RightParen, ")".to_string(), 1),
                Token::new(TokenKind::LeftBrace, "{".to_string(), 1),
                Token::new(TokenKind::RightBrace, "}".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    #[test]
    fn operator_works() {
        let tokens: Result<Vec<_>> = ScanTokens::new("!*+-/=<> <= ==").collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::new(TokenKind::Bang, "!".to_string(), 1),
                Token::new(TokenKind::Star, "*".to_string(), 1),
                Token::new(TokenKind::Plus, "+".to_string(), 1),
                Token::new(TokenKind::Minus, "-".to_string(), 1),
                Token::new(TokenKind::Slash, "/".to_string(), 1),
                Token::new(TokenKind::Equal, "=".to_string(), 1),
                Token::new(TokenKind::Less, "<".to_string(), 1),
                Token::new(TokenKind::Greater, ">".to_string(), 1),
                Token::new(TokenKind::LessEqual, "<=".to_string(), 1),
                Token::new(TokenKind::EqualEqual, "==".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    #[test]
    fn string_works() {
        let tokens: Result<Vec<_>> = ScanTokens::new(r#""+ -""#).collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::new(TokenKind::String, "+ -".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }
}
