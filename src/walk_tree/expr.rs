use super::token::Token;

pub type Operator = Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Operator, Box<Expr>),
    Unary(Operator, Box<Expr>),
    Literal(Literal),
    Grouping(Box<Expr>),
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Self::Literal(Literal::Boolean(value))
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Self::Literal(Literal::Number(value))
    }
}

impl From<()> for Expr {
    fn from(_value: ()) -> Self {
        Self::Literal(Literal::Nil)
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Self::Literal(Literal::String(value))
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Self::Literal(Literal::String(String::from(value)))
    }
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}
