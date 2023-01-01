use super::token_kind::TokenKind;

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize) -> Self {
        Self { kind, lexeme, line }
    }
}
