use super::{token::Token, value::Value};

pub type Operator = Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Operator, Box<Expr>),
    Unary(Operator, Box<Expr>),
    Literal(Value),
    Grouping(Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Variable(Token),
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Self::Literal(Value::Boolean(value))
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Self::Literal(Value::Number(value))
    }
}

impl From<()> for Expr {
    fn from(_value: ()) -> Self {
        Self::Literal(Value::Nil)
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Self::Literal(Value::String(value))
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Self::Literal(Value::String(String::from(value)))
    }
}
