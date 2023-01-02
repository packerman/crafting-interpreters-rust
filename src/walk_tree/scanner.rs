use anyhow::Result;

use super::{error, token::Token, token_kind::TokenKind};

pub struct ScanTokens {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    consumed: bool,
}

impl ScanTokens {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            consumed: false,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Option<Result<Token>> {
        let ch = self.advance();
        match ch {
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
            '!' => self.cond_emit('=', TokenKind::BangEqual, TokenKind::Bang),
            '=' => self.cond_emit('=', TokenKind::EqualEqual, TokenKind::Equal),
            '<' => self.cond_emit('=', TokenKind::LessEqual, TokenKind::Less),
            '>' => self.cond_emit('=', TokenKind::GreaterEqual, TokenKind::Greater),
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else {
                    self.emit_token(TokenKind::Slash)
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
            ch => {
                if ch.is_ascii_digit() {
                    self.number()
                } else {
                    Some(error::error(self.line, "Unexpected character"))
                }
            }
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
            self.emit_token(TokenKind::String(value))
        }
    }

    fn number(&mut self) -> Option<Result<Token>> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
        }
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        let value = str::parse(&self.current_lexeme()).expect("Expected valid number");
        self.emit_token(TokenKind::Number(value))
    }

    fn start(&mut self) {
        self.start = self.current;
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
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

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn emit_token(&self, kind: TokenKind) -> Option<Result<Token>> {
        Some(Ok(Token::new(kind, self.current_lexeme(), self.line)))
    }

    fn cond_emit(
        &mut self,
        if_match: char,
        then_emit: TokenKind,
        else_emit: TokenKind,
    ) -> Option<Result<Token>> {
        let token_kind = if self.match_char(if_match) {
            then_emit
        } else {
            else_emit
        };
        self.emit_token(token_kind)
    }

    fn copy_slice(&self, begin: usize, end: usize) -> String {
        String::from_iter(&self.source[begin..end])
    }

    fn current_lexeme(&self) -> String {
        self.copy_slice(self.start, self.current)
    }

    fn emit_eof(&mut self) -> Option<Result<Token>> {
        if self.consumed {
            None
        } else {
            self.consumed = true;
            self.emit_token(TokenKind::Eof)
        }
    }
}

impl Iterator for ScanTokens {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.start();
            if self.is_at_end() {
                return self.emit_eof();
            }
            let token = self.scan_token();
            if token.is_some() {
                return token;
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
                Token::new(
                    TokenKind::String("+ -".to_string()),
                    "\"+ -\"".to_string(),
                    1
                ),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    #[test]
    fn numbers_works() {
        let tokens: Result<Vec<_>> = ScanTokens::new("3.14 + 1").collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::new(TokenKind::Number(3.14), "3.14".to_string(), 1),
                Token::new(TokenKind::Plus, "+".to_string(), 1),
                Token::new(TokenKind::Number(1.0), "1".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }
}
