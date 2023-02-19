use std::rc::Rc;

use super::{stmt::Stmt, token::Token, value::Cell};

pub type Operator = Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Operator, Box<Expr>),
    Call(Box<Expr>, Token, Box<[Box<Expr>]>),
    Unary(Operator, Box<Expr>),
    Literal(Cell),
    Grouping(Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Variable(Token),
    Assignment(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Function(Option<Token>, Rc<[Token]>, Rc<[Box<Stmt>]>),
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

impl From<Rc<str>> for Expr {
    fn from(value: Rc<str>) -> Self {
        Self::Literal(Cell::from(value))
    }
}

impl From<()> for Expr {
    fn from(_value: ()) -> Self {
        Self::Literal(Cell::from(()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn identity_exprs_keys() {
        let e1: *const Expr = &Expr::from(2.0);
        let e2: *const Expr = &Expr::from(2.0);
        let e3: *const Expr = &Expr::from(3.0);
        let mut m = HashMap::new();
        m.insert(e1, 1);
        m.insert(e2, 2);
        m.insert(e3, 3);
        assert_eq!(m[&e1], 1);
        assert_eq!(m[&e2], 2);
        assert_eq!(m[&e3], 3);
    }
}
