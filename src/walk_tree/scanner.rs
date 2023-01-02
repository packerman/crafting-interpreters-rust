use anyhow::Result;

use super::{error, token::Token, token_kind::TokenKind};

pub struct ScanTokens {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
}

impl ScanTokens {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Option<Result<Token>> {
        let ch = self.advance();
        match ch {
            '(' => Some(self.new_token(TokenKind::LeftParen)),
            ')' => Some(self.new_token(TokenKind::RightParen)),
            '{' => Some(self.new_token(TokenKind::LeftBrace)),
            '}' => Some(self.new_token(TokenKind::RightBrace)),
            ',' => Some(self.new_token(TokenKind::Comma)),
            '.' => Some(self.new_token(TokenKind::Dot)),
            '-' => Some(self.new_token(TokenKind::Minus)),
            '+' => Some(self.new_token(TokenKind::Plus)),
            ';' => Some(self.new_token(TokenKind::Semicolon)),
            '*' => Some(self.new_token(TokenKind::Star)),
            '!' => {
                let token_kind = if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                Some(self.new_token(token_kind))
            }
            '=' => {
                let token_kind = if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                Some(self.new_token(token_kind))
            }
            '<' => {
                let token_kind = if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                Some(self.new_token(token_kind))
            }
            '>' => {
                let token_kind = if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                Some(self.new_token(token_kind))
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else {
                    Some(self.new_token(TokenKind::Slash))
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
    }

    fn string(&mut self) -> Option<Result<Token>> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            Some(error::error(self.line, "Unterminated string"))
        } else {
            self.advance();
            let value = self.copy_slice(self.start + 1, self.current - 1);
            Some(self.new_token(TokenKind::String(value)))
        }
    }

    fn start(&mut self) {
        self.start = self.current;
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn new_token(&self, kind: TokenKind) -> Result<Token> {
        Ok(Token::new(
            kind,
            self.copy_slice(self.start, self.current),
            self.line,
        ))
    }

    fn copy_slice(&self, begin: usize, end: usize) -> String {
        String::from_iter(&self.source[begin..end])
    }
}

impl Iterator for ScanTokens {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_end() {
            return Some(self.new_token(TokenKind::Eof));
        }
        self.start();
        self.scan_token()
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
                Token::new(TokenKind::String("+ -".to_string()), "+ -".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }
}
