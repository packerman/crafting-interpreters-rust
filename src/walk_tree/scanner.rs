use std::collections::HashMap;

use super::{error::ErrorReporter, token::Token, token_kind::TokenKind};

pub struct Scanner<'a> {
    keywords: HashMap<&'a str, TokenKind>,
    error_reporter: &'a ErrorReporter,
}

impl<'a> Scanner<'a> {
    pub fn new(error_reporter: &'a ErrorReporter) -> Self {
        Self {
            keywords: Self::keywords(),
            error_reporter,
        }
    }

    pub fn scan_tokens(&self, source: &str) -> ScanTokens {
        ScanTokens::new(source, &self.keywords, self.error_reporter)
    }

    fn keywords() -> HashMap<&'a str, TokenKind> {
        HashMap::from([
            ("and", TokenKind::And),
            ("class", TokenKind::Class),
            ("else", TokenKind::Else),
            ("false", TokenKind::False),
            ("for", TokenKind::For),
            ("fun", TokenKind::Fun),
            ("if", TokenKind::If),
            ("nil", TokenKind::Nil),
            ("or", TokenKind::Or),
            ("print", TokenKind::Print),
            ("return", TokenKind::Return),
            ("super", TokenKind::Super),
            ("this", TokenKind::This),
            ("true", TokenKind::True),
            ("var", TokenKind::Var),
            ("while", TokenKind::While),
        ])
    }
}

pub struct ScanTokens<'a> {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    consumed: bool,
    keywords: &'a HashMap<&'a str, TokenKind>,
    error_reporter: &'a ErrorReporter,
}

impl<'a> ScanTokens<'a> {
    pub fn new(
        source: &str,
        keywords: &'a HashMap<&'a str, TokenKind>,
        error_reporter: &'a ErrorReporter,
    ) -> Self {
        Self {
            source: source.chars().collect(),
            keywords,
            start: 0,
            current: 0,
            line: 1,
            consumed: false,
            error_reporter,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Option<Token> {
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
                    self.comment()
                } else if self.match_char('*') {
                    self.block_comment()
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
                } else if ch.is_ascii_alphabetic() {
                    self.identifier()
                } else {
                    self.error_reporter.error(self.line, "Unexpected character");
                    None
                }
            }
        }
    }

    fn comment(&mut self) -> Option<Token> {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
        None
    }

    fn block_comment(&mut self) -> Option<Token> {
        let mut nest = 1;
        while !self.is_at_end() && nest > 0 {
            let ch = self.advance();
            if ch == '\n' {
                self.line += 1;
            }
            if ch == '/' && self.match_char('*') {
                nest += 1;
            } else if ch == '*' && self.match_char('/') {
                nest -= 1;
            }
        }
        if nest > 0 {
            self.error_reporter.error(self.line, "Unexpected EOF");
        }
        None
    }

    fn string(&mut self) -> Option<Token> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.error_reporter.error(self.line, "Unterminated string");
            None
        } else {
            self.advance();
            let value = self.copy_slice(self.start + 1, self.current - 1);
            self.emit_token(TokenKind::String(value))
        }
    }

    fn number(&mut self) -> Option<Token> {
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

    fn identifier(&mut self) -> Option<Token> {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }
        let text = self.current_lexeme();
        self.emit_token(
            self.keywords
                .get(text.as_str())
                .cloned()
                .unwrap_or(TokenKind::Identifier),
        )
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

    fn emit_token(&self, kind: TokenKind) -> Option<Token> {
        Some(Token::new(kind, self.current_lexeme(), self.line))
    }

    fn cond_emit(
        &mut self,
        if_match: char,
        then_emit: TokenKind,
        else_emit: TokenKind,
    ) -> Option<Token> {
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

    fn emit_eof(&mut self) -> Option<Token> {
        if self.consumed {
            None
        } else {
            self.consumed = true;
            self.emit_token(TokenKind::Eof)
        }
    }
}

impl<'a> Iterator for ScanTokens<'a> {
    type Item = Token;

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
        let tokens = self::scan_tokens("// this is a comment");
        assert_eq!(tokens, vec![Token::new(TokenKind::Eof, "".to_string(), 1)])
    }

    #[test]
    fn grouping_stuff_works() {
        let tokens = self::scan_tokens("(( )){}");
        assert_eq!(
            tokens,
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
        let tokens = self::scan_tokens("!*+-/=<> <= ==");
        assert_eq!(
            tokens,
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
        let tokens = self::scan_tokens(r#""+ -""#);
        assert_eq!(
            tokens,
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
        let tokens = self::scan_tokens("3.14 + 1");
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Number(3.14), "3.14".to_string(), 1),
                Token::new(TokenKind::Plus, "+".to_string(), 1),
                Token::new(TokenKind::Number(1.0), "1".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    #[test]
    fn identifier_works() {
        let tokens = self::scan_tokens("and andaluzja and aluzja And");
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::And, "and".to_string(), 1),
                Token::new(TokenKind::Identifier, "andaluzja".to_string(), 1),
                Token::new(TokenKind::And, "and".to_string(), 1),
                Token::new(TokenKind::Identifier, "aluzja".to_string(), 1),
                Token::new(TokenKind::Identifier, "And".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    #[test]
    fn block_comment_works() {
        let tokens = self::scan_tokens("a /* x y */ b");
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, "a".to_string(), 1),
                Token::new(TokenKind::Identifier, "b".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    #[test]
    fn nested_block_comment_works() {
        let tokens = self::scan_tokens("a /* /* x */ y */ b");
        assert_eq!(
            tokens,
            vec![
                Token::new(TokenKind::Identifier, "a".to_string(), 1),
                Token::new(TokenKind::Identifier, "b".to_string(), 1),
                Token::new(TokenKind::Eof, "".to_string(), 1)
            ]
        )
    }

    fn scan_tokens(source: &str) -> Vec<Token> {
        let error_reporter = ErrorReporter::new();
        Scanner::new(&error_reporter).scan_tokens(source).collect()
    }
}
