use super::token::Token;

pub type Operator = Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Operator, Box<Expr>),
    Unary(Operator, Box<Expr>),
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Grouping(Box<Expr>),
}
