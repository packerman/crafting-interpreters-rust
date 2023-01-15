use super::{token::Token, value::Cell};

pub type Operator = Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Operator, Box<Expr>),
    Unary(Operator, Box<Expr>),
    Literal(Cell),
    Grouping(Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Variable(Token),
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Self::Literal(Cell::from(value))
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Self::Literal(Cell::from(value))
    }
}

impl From<()> for Expr {
    fn from(_value: ()) -> Self {
        Self::Literal(Cell::from(()))
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Self::Literal(Cell::from(value))
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Self::Literal(Cell::from(String::from(value)))
    }
}
