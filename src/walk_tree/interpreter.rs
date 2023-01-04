use anyhow::Result;

use crate::walk_tree::error::RuntimeError;

use super::{
    error::ErrorReporter,
    expr::Expr,
    token::Token,
    token_kind::TokenKind,
    value::{self, Value},
};

pub struct Interpreter<'a> {
    error_repoter: &'a ErrorReporter,
}

impl<'a> Interpreter<'a> {
    pub fn new(error_repoter: &'a ErrorReporter) -> Self {
        Self { error_repoter }
    }

    pub fn interpret(&self, expr: &Expr) {
        let result = self.interpret_expr(expr);
        match &result {
            Ok(value) => println!("{}", value),
            Err(error) => self.error_repoter.runtime_error(error),
        }
    }
    fn interpret_expr(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(literal) => self.interpret_literal(literal),
            Expr::Grouping(expr) => self.interpret_expr(expr),
            Expr::Unary(operator, operand) => self.interpret_unary(operator, operand),
            Expr::Binary(left, operator, right) => self.interpret_binary(left, operator, right),
        }
    }

    fn interpret_literal(&self, literal: &Value) -> Result<Value, RuntimeError> {
        Ok(literal.to_owned())
    }

    fn interpret_unary(&self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let right = self.interpret_expr(right)?;
        Ok(match operator.kind {
            TokenKind::Minus => {
                self.check_number_operand(operator, &right)?;
                value::unary_operation::<f64>(|a| -a, right)
            }
            TokenKind::Bang => Value::from(!right.is_truthy()),
            _ => unreachable!(),
        })
    }

    fn interpret_binary(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left = self.interpret_expr(left)?;
        let right = self.interpret_expr(right)?;
        Ok(match operator.kind {
            TokenKind::Minus => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation::<f64>(|a, b| a + b, left, right)
            }
            TokenKind::Plus => {
                if left.is_number() && right.is_number() {
                    value::binary_operation::<f64>(|a, b| a + b, left, right)
                } else if left.is_string() && right.is_string() {
                    value::binary_operation::<String>(|a, b| a + &b, left, right)
                } else {
                    Err(RuntimeError::new(
                        operator.to_owned(),
                        "Operands must be two numbers or two string.",
                    ))?
                }
            }
            TokenKind::Slash => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation::<f64>(|a, b| a / b, left, right)
            }
            TokenKind::Star => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation::<f64>(|a, b| a * b, left, right)
            }
            TokenKind::Greater => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a > b, left, right)
            }
            TokenKind::GreaterEqual => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a >= b, left, right)
            }
            TokenKind::Less => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a < b, left, right)
            }
            TokenKind::LessEqual => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a <= b, left, right)
            }
            TokenKind::BangEqual => Value::from(left != right),
            TokenKind::EqualEqual => Value::from(left == right),
            _ => unreachable!(),
        })
    }

    fn check_number_operand(&self, operator: &Token, operand: &Value) -> Result<(), RuntimeError> {
        if operand.is_number() {
            Ok(())
        } else {
            Err(RuntimeError::new(
                operator.to_owned(),
                "Operand must be a number.",
            ))
        }
    }

    fn check_number_operands(
        &self,
        operator: &Token,
        left: &Value,
        right: &Value,
    ) -> Result<(), RuntimeError> {
        if left.is_number() && right.is_number() {
            Ok(())
        } else {
            Err(RuntimeError::new(
                operator.to_owned(),
                "Operand must be numbers.",
            ))
        }
    }
}
