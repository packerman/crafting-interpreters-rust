use super::{
    expr::Expr,
    token::Token,
    token_kind::TokenKind,
    value::{self, Value},
};

pub fn interpret(expr: &Expr) -> Value {
    match expr {
        Expr::Literal(literal) => interpret_literal(literal),
        Expr::Grouping(expr) => interpret(expr),
        Expr::Unary(operator, operand) => interpret_unary(operator, operand),
        Expr::Binary(left, operator, right) => interpret_binary(left, operator, right),
    }
}

pub fn interpret_literal(literal: &Value) -> Value {
    literal.to_owned()
}

pub fn interpret_unary(operator: &Token, right: &Expr) -> Value {
    let right = interpret(right);
    match operator.kind {
        TokenKind::Minus => value::unary_operation::<f64>(|a| -a, right),
        TokenKind::Bang => Value::from(!right.is_truthy()),
        _ => unreachable!(),
    }
}

pub fn interpret_binary(left: &Expr, operator: &Token, right: &Expr) -> Value {
    let left = interpret(left);
    let right = interpret(right);
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
        TokenKind::BangEqual => value::binary_relation::<f64>(|a, b| a != b, left, right),
        TokenKind::EqualEqual => value::binary_relation::<f64>(|a, b| a == b, left, right),
        _ => unreachable!(),
    }
}
