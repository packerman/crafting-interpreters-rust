use crate::walk_tree::expr::Expr;

use super::token::Token;

pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Box<Expr>),
    Print(Box<Expr>),
    VarDeclaration(Token, Option<Box<Expr>>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
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
