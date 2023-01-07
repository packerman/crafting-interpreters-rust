use anyhow::Result;

use crate::walk_tree::error::RuntimeError;
use crate::walk_tree::stmt::Stmt;

use super::{
    error::ErrorReporter,
    expr::Expr,
    token::{Token, TokenKind},
    value::{self, Value},
};

pub struct Interpreter<'a> {
    error_reporter: &'a ErrorReporter,
}

impl<'a> Interpreter<'a> {
    pub fn new(error_reporter: &'a ErrorReporter) -> Self {
        Self { error_reporter }
    }

    pub fn interpret(&self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary(operator, operand) => self.evaluate_unary(operator, operand),
            Expr::Binary(left, operator, right) => self.evaluate_binary(left, operator, right),
            Expr::Ternary(condition, then_expr, else_expr) => {
                self.evaluate_ternary(condition, then_expr, else_expr)
            }
        }
    }

    fn execute(&self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => self.execute_expression_stmt(expr),
            Stmt::Print(expr) => self.execute_print_stmt(expr),
        }
    }

    fn execute_expression_stmt(&self, expr: &Expr) -> Result<(), RuntimeError> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn execute_print_stmt(&self, expr: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn evaluate_literal(&self, literal: &Value) -> Result<Value, RuntimeError> {
        Ok(literal.to_owned())
    }

    fn evaluate_unary(&self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let right = self.evaluate(right)?;
        match operator.kind {
            TokenKind::Minus => {
                self.check_number_operand(operator, &right)?;
                value::unary_operation::<f64>(|a| -a, operator, right)
            }
            TokenKind::Bang => Ok(Value::from(!right.is_truthy())),
            _ => unreachable!(),
        }
    }

    fn evaluate_binary(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match operator.kind {
            TokenKind::Minus => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation::<f64>(|a, b| a + b, left, operator, right)
            }
            TokenKind::Plus => {
                if left.is_number() && right.is_number() {
                    value::binary_operation::<f64>(|a, b| a + b, left, operator, right)
                } else if left.is_string() && right.is_string() {
                    value::binary_operation::<String>(|a, b| a + &b, left, operator, right)
                } else {
                    Err(RuntimeError::new(
                        operator.to_owned(),
                        "Operands must be two numbers or two string.",
                    ))?
                }
            }
            TokenKind::Slash => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation::<f64>(|a, b| a / b, left, operator, right)
            }
            TokenKind::Star => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation::<f64>(|a, b| a * b, left, operator, right)
            }
            TokenKind::Greater => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a > b, left, operator, right)
            }
            TokenKind::GreaterEqual => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a >= b, left, operator, right)
            }
            TokenKind::Less => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a < b, left, operator, right)
            }
            TokenKind::LessEqual => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_relation::<f64>(|a, b| a <= b, left, operator, right)
            }
            TokenKind::BangEqual => Ok(Value::from(left != right)),
            TokenKind::EqualEqual => Ok(Value::from(left == right)),
            _ => unreachable!(),
        }
    }

    fn evaluate_ternary(
        &self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> Result<Value, RuntimeError> {
        let condition = self.evaluate(condition)?;
        if condition.is_truthy() {
            self.evaluate(then_expr)
        } else {
            self.evaluate(else_expr)
        }
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

#[cfg(test)]
mod tests {
    use crate::walk_tree::{parser::Parser, scanner::Scanner};

    use super::*;

    #[test]
    fn arithmetic() {
        assert_evaluates_to("2;", 2.0);
        assert_evaluates_to("2+3;", 5.0);
        assert_evaluates_to("2*3;", 6.0);
        assert_evaluates_to("1+2*3;", 7.0);
        assert_evaluates_to("(1+2)*3;", 9.0);
        assert_evaluates_to("1*2+3;", 5.0);
        assert_evaluates_to("1+2+3;", 6.0);
    }

    #[test]
    fn comparison_works() {
        assert_evaluates_to("2 == 2;", true);
        assert_evaluates_to("2 != 2;", false);
        assert_evaluates_to("2 < 2;", false);
        assert_evaluates_to("2 <= 2;", true);
        assert_evaluates_to("2 > 2;", false);
        assert_evaluates_to("2 >= 2;", true);
        assert_evaluates_to("2 == 3;", false);
        assert_evaluates_to("2 != 3;", true);
        assert_evaluates_to("2 < 3;", true);
        assert_evaluates_to("2 <= 3;", true);
        assert_evaluates_to("2 > 3;", false);
        assert_evaluates_to("2 >= 3;", false);
    }

    #[test]
    fn ternary_works() {
        assert_evaluates_to("2 < 3 ? 2 * 3 : 2 + 3;", 6.0);
        assert_evaluates_to("2 > 3 ? 2 * 3 : 2 + 3;", 5.0);
    }

    #[test]
    fn concat_string_works() {
        assert_evaluates_to(r#""ala" + " ma " + "kota";"#, "ala ma kota");
    }

    fn assert_evaluates_to<T>(source: &str, value: T)
    where
        Value: From<T>,
    {
        assert_eq!(test_interpret_expr(source).unwrap(), Value::from(value))
    }

    fn test_interpret_expr(source: &str) -> Option<Value> {
        let error_reporter = ErrorReporter::new();
        let scanner = Scanner::new(&error_reporter);
        let tokens: Vec<_> = scanner.scan_tokens(source).collect();
        let mut parser = Parser::new(tokens, &error_reporter);
        let tree = parser.parse()?;
        let expr = tree[0].as_expr().unwrap();
        let interpreter = Interpreter::new(&error_reporter);
        interpreter.evaluate(expr).ok()
    }
}
