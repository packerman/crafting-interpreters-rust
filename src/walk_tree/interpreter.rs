use anyhow::Result;
use std::io::Write;
use std::io::{self, Stdout};
use std::sync::Arc;

use crate::walk_tree::error::RuntimeError;
use crate::walk_tree::stmt::Stmt;

use super::environment::Environment;
use super::{
    error::ErrorReporter,
    expr::Expr,
    token::{Token, TokenKind},
    value::{self, Cell},
};

pub struct Interpreter<'a, W> {
    error_reporter: &'a ErrorReporter,
    output: W,
    environment: Environment,
}

impl<'a> Interpreter<'a, Stdout> {
    pub fn new(error_reporter: &'a ErrorReporter) -> Self {
        Self::new_with_output(error_reporter, io::stdout())
    }
}

impl<'a, W> Interpreter<'a, W>
where
    W: Write,
{
    pub fn new_with_output(error_reporter: &'a ErrorReporter, output: W) -> Self {
        Self {
            error_reporter,
            output,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                self.error_reporter.runtime_error(&error);
                return;
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Cell, RuntimeError> {
        match expr {
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary(operator, operand) => self.evaluate_unary(operator, operand),
            Expr::Binary(left, operator, right) => self.evaluate_binary(left, operator, right),
            Expr::Ternary(condition, then_expr, else_expr) => {
                self.evaluate_ternary(condition, then_expr, else_expr)
            }
            Expr::Variable(name) => self.environment.get(name).map(|value| value.to_owned()),
            Expr::Assignment(name, expr) => self.execute_assign_expr(name, expr),
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => self.execute_expression_stmt(expr),
            Stmt::Print(expr) => self.execute_print_stmt(expr),
            Stmt::VarDeclaration(name, initializer) => {
                self.execute_var_stmt(name, initializer.as_deref())
            }
        }
    }

    fn execute_expression_stmt(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn execute_print_stmt(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr)?;
        writeln!(self.output, "{}", value)
            .map_err(|err| RuntimeError::from(format!("Print error: {}", err)))
    }

    fn execute_var_stmt(
        &mut self,
        name: &Token,
        initializer: Option<&Expr>,
    ) -> Result<(), RuntimeError> {
        let value = if let Some(initializer) = initializer {
            self.evaluate(initializer)?
        } else {
            Cell::from(())
        };
        self.environment.define(name, value);
        Ok(())
    }

    fn execute_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<Cell, RuntimeError> {
        let value = self.evaluate(expr)?;
        self.environment.assign(name, value.to_owned())?;
        Ok(value)
    }

    fn evaluate_literal(&self, literal: &Cell) -> Result<Cell, RuntimeError> {
        Ok(literal.to_owned())
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<Cell, RuntimeError> {
        let right = self.evaluate(right)?;
        match operator.kind {
            TokenKind::Minus => {
                self.check_number_operand(operator, &right)?;
                value::unary_operation(|a: f64| -a, operator, right)
            }
            TokenKind::Bang => Ok(Cell::from(!right.is_truthy())),
            _ => unreachable!(),
        }
    }

    fn evaluate_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Cell, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        match operator.kind {
            TokenKind::Minus => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b: f64| a + b, left, operator, right)
            }
            TokenKind::Plus => {
                if left.is_number() && right.is_number() {
                    value::binary_operation(|a: f64, b: f64| a + b, left, operator, right)
                } else if left.is_string() && right.is_string() {
                    value::binary_operation(
                        |a: String, b: Arc<str>| Arc::from(a.to_owned() + &b),
                        left,
                        operator,
                        right,
                    )
                } else {
                    Err(RuntimeError::new(
                        operator.to_owned(),
                        "Operands must be two numbers or two string.",
                    ))?
                }
            }
            TokenKind::Slash => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b: f64| a / b, left, operator, right)
            }
            TokenKind::Star => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b: f64| a * b, left, operator, right)
            }
            TokenKind::Greater => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b| a > b, left, operator, right)
            }
            TokenKind::GreaterEqual => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b| a >= b, left, operator, right)
            }
            TokenKind::Less => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b| a < b, left, operator, right)
            }
            TokenKind::LessEqual => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b| a <= b, left, operator, right)
            }
            TokenKind::BangEqual => Ok(Cell::from(left != right)),
            TokenKind::EqualEqual => Ok(Cell::from(left == right)),
            _ => unreachable!(),
        }
    }

    fn evaluate_ternary(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> Result<Cell, RuntimeError> {
        let condition = self.evaluate(condition)?;
        if condition.is_truthy() {
            self.evaluate(then_expr)
        } else {
            self.evaluate(else_expr)
        }
    }

    fn check_number_operand(&self, operator: &Token, operand: &Cell) -> Result<(), RuntimeError> {
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
        left: &Cell,
        right: &Cell,
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
    use std::sync::Arc;

    use crate::walk_tree::{parser::Parser, scanner::Scanner};
    use anyhow::Context;

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
        assert_evaluates_to(r#""ala" + " ma " + "kota";"#, Arc::from("ala ma kota"));
    }

    #[test]
    fn print_works() {
        assert_prints(r#"print 2+3;"#, b"5\n");
    }

    #[test]
    fn define_var_works() {
        assert_prints(
            r#"
        var a = 1;
        var b = 2;
        print a + b;"#,
            b"3\n",
        );
    }

    #[test]
    fn assignment_works() {
        assert_prints(
            r#"
        var a = 1;
        print a;
        a = 2;
        print a;
        "#,
            b"1\n2\n",
        );
    }

    fn assert_evaluates_to<T>(source: &str, value: T)
    where
        Cell: From<T>,
    {
        assert_eq!(test_interpret_stmt_expr(source).unwrap(), Cell::from(value))
    }

    fn assert_prints(source: &str, value: &[u8]) {
        assert_eq!(test_interpreter_output(source).unwrap(), value)
    }

    fn test_interpreter_output(source: &str) -> Result<Vec<u8>> {
        let error_reporter = ErrorReporter::new();
        let tree = test_parse(source, &error_reporter).context("Error in parsing")?;
        let mut output = Vec::new();
        let mut interpreter = Interpreter::new_with_output(&error_reporter, &mut output);
        interpreter.interpret(&tree);
        Ok(output)
    }

    fn test_interpret_stmt_expr(source: &str) -> Result<Cell> {
        let error_reporter = ErrorReporter::new();
        let tree = test_parse(source, &error_reporter).context("Parse error")?;
        let expr = tree[0].as_expr().unwrap();
        let mut interpreter = Interpreter::new(&error_reporter);
        interpreter.evaluate(expr).context("Evaluating error")
    }

    fn test_parse(source: &str, error_reporter: &ErrorReporter) -> Option<Vec<Stmt>> {
        let scanner = Scanner::new(error_reporter);
        let tokens: Vec<_> = scanner.scan_tokens(source).collect();
        let mut parser = Parser::new(tokens, error_reporter);
        parser.parse()
    }
}
