use std::rc::Rc;

use crate::walk_tree::expr::Expr;

use super::token::Token;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Block(Rc<[Box<Stmt>]>),
    Expr(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Return(Token, Option<Box<Expr>>),
    While(Box<Expr>, Box<Stmt>),
    VarDeclaration(Token, Option<Box<Expr>>),
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
