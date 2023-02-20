use std::rc::Rc;

use super::{stmt::Stmt, token::Token, value::Cell};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Box<[Box<Expr>]>,
    },
    Unary {
        operator: Token,
        operand: Box<Expr>,
    },
    Literal(Cell),
    Grouping(Box<Expr>),
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Variable(Token),
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Function(Function),
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
}

impl Expr {
    pub fn binary(left: Box<Expr>, operator: Token, right: Box<Expr>) -> Self {
        Self::Binary {
            left,
            operator,
            right,
        }
    }

    pub fn function(name: Option<Token>, parameters: Rc<[Token]>, body: Rc<[Box<Stmt>]>) -> Self {
        Self::Function(Function::new(name, parameters, body))
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    name: Option<Token>,
    parameters: Rc<[Token]>,
    body: Rc<[Box<Stmt>]>,
}

impl Function {
    pub fn new(name: Option<Token>, parameters: Rc<[Token]>, body: Rc<[Box<Stmt>]>) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }

    pub fn name(&self) -> Option<&Token> {
        self.name.as_ref()
    }

    pub fn parameters(&self) -> &Rc<[Token]> {
        &self.parameters
    }

    pub fn body(&self) -> &Rc<[Box<Stmt>]> {
        &self.body
    }
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

impl From<Function> for Expr {
    fn from(value: Function) -> Self {
        Self::Function(value)
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
