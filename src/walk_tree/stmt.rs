use std::rc::Rc;

use crate::walk_tree::expr::Expr;

use super::token::Token;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Block(Rc<[Box<Stmt>]>),
    Expr(Box<Expr>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Return {
        keyword: Token,
        expr: Option<Box<Expr>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    VarDeclaration {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
}

impl Stmt {
    pub fn as_expr(&self) -> Option<&Expr> {
        if let Self::Expr(expr) = self {
            Some(expr)
        } else {
            None
        }
    }
}
