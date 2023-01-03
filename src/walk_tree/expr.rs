use super::token::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
}
