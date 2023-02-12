use std::sync::Arc;

use crate::walk_tree::expr::Expr;

use super::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Block(Arc<[Box<Stmt>]>),
    Expr(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Function(Token, Arc<[Token]>, Arc<[Box<Stmt>]>),
    Print(Box<Expr>),
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
