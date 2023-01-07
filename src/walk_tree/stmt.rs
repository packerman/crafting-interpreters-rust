use crate::walk_tree::expr::Expr;

pub enum Stmt {
    Expr(Box<Expr>),
    Print(Box<Expr>),
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
