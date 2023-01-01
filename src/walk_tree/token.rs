use super::token_kind::TokenKind;

#[derive(Debug, PartialEq)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize) -> Self {
        Self { kind, lexeme, line }
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}
