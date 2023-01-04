use super::{
    expr::Expr,
    token::Token,
    token_kind::TokenKind,
    value::{self, Value},
};

pub fn interpret(expr: &Expr) -> RValue {}

fn interpret_expr(expr: &Expr) -> Value {
    match expr {
        Expr::Literal(literal) => interpret_literal(literal),
        Expr::Grouping(expr) => interpret_expr(expr),
        Expr::Unary(operator, operand) => interpret_unary(operator, operand),
        Expr::Binary(left, operator, right) => interpret_binary(left, operator, right),
    }
}

fn interpret_literal(literal: &Value) -> Value {
    literal.to_owned()
}

fn interpret_unary(operator: &Token, right: &Expr) -> Value {
    let right = interpret_expr(right);
    match operator.kind {
        TokenKind::Minus => value::unary_operation::<f64>(|a| -a, right),
        TokenKind::Bang => Value::from(!right.is_truthy()),
        _ => unreachable!(),
    }
}

fn interpret_binary(left: &Expr, operator: &Token, right: &Expr) -> Value {
    let left = interpret_expr(left);
    let right = interpret_expr(right);
    match operator.kind {
        TokenKind::Minus => value::binary_operation::<f64>(|a, b| a + b, left, right),
        TokenKind::Plus => {
            if left.is_number() && right.is_number() {
                value::binary_operation::<f64>(|a, b| a + b, left, right)
            } else if left.is_string() && right.is_string() {
                value::binary_operation::<String>(|a, b| a + &b, left, right)
            } else {
                unreachable!()
            }
        }
        TokenKind::Slash => value::binary_operation::<f64>(|a, b| a / b, left, right),
        TokenKind::Star => value::binary_operation::<f64>(|a, b| a * b, left, right),
        TokenKind::Greater => value::binary_relation::<f64>(|a, b| a > b, left, right),
        TokenKind::GreaterEqual => value::binary_relation::<f64>(|a, b| a >= b, left, right),
        TokenKind::Less => value::binary_relation::<f64>(|a, b| a < b, left, right),
        TokenKind::LessEqual => value::binary_relation::<f64>(|a, b| a <= b, left, right),
        TokenKind::BangEqual => Value::from(left != right),
        TokenKind::EqualEqual => Value::from(left == right),
        _ => unreachable!(),
    }
}
