use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::io::Write;
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
    global_environment: Arc<RefCell<Environment>>,
}

impl<'a, W> Interpreter<'a, W>
where
    W: Write,
{
    pub fn new_with_output(error_reporter: &'a ErrorReporter, output: W) -> Self {
        Self {
            error_reporter,
            output,
            global_environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Box<Stmt>]) {
        let env = Arc::clone(&self.global_environment);
        for statement in statements {
            if let Err(error) = self.execute(statement, &env) {
                self.error_reporter.runtime_error(&error);
                return;
            }
        }
    }

    fn evaluate(
        &mut self,
        expr: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<Cell, RuntimeError> {
        match expr {
            Expr::Literal(literal) => self.evaluate_literal(literal),
            Expr::Grouping(expr) => self.evaluate(expr, env),
            Expr::Unary(operator, operand) => self.evaluate_unary(operator, operand, env),
            Expr::Binary(left, operator, right) => self.evaluate_binary(left, operator, right, env),
            Expr::Logical(left, operator, right) => {
                self.evaluate_logical(left, operator, right, env)
            }
            Expr::Ternary(condition, then_expr, else_expr) => {
                self.evaluate_ternary(condition, then_expr, else_expr, env)
            }
            Expr::Variable(name) => env.borrow().get(name),
            Expr::Assignment(name, expr) => self.execute_assign_expr(name, expr, env),
        }
    }

    pub fn evaluate_and_print(&mut self, expr: &Expr) -> Result<Cell> {
        let result = self.evaluate(expr, &Arc::clone(&self.global_environment));
        match &result {
            Ok(result) => {
                writeln!(self.output, "{result}")?;
            }
            Err(error) => self.error_reporter.runtime_error(error),
        }
        result.map_err(|err| anyhow!("Evaluate error: {}", err))
    }

    fn execute(
        &mut self,
        stmt: &Stmt,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block(stmts) => self.execute_block_stmt(stmts, env),
            Stmt::Expr(expr) => self.execute_expression_stmt(expr, env),
            Stmt::If(condition, then_branch, else_branch) => {
                self.execute_if_stmt(condition, then_branch, else_branch.as_deref(), env)
            }
            Stmt::Print(expr) => self.execute_print_stmt(expr, env),
            Stmt::While(condition, body) => self.execute_while_stmt(condition, body, env),
            Stmt::VarDeclaration(name, initializer) => {
                self.execute_var_stmt(name, initializer.as_deref(), env)
            }
        }
    }

    fn execute_block_stmt(
        &mut self,
        statements: &[Box<Stmt>],
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let environment = Environment::new_with_enclosing(Arc::clone(env));
        for statement in statements {
            self.execute(statement, &environment)?;
        }
        Ok(())
    }

    fn execute_expression_stmt(
        &mut self,
        expr: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        self.evaluate(expr, env)?;
        Ok(())
    }

    fn execute_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        if self.evaluate(condition, env)?.is_truthy() {
            self.execute(then_branch, env)?
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch, env)?
        }
        Ok(())
    }

    fn execute_print_stmt(
        &mut self,
        expr: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr, env)?;
        writeln!(self.output, "{value}")
            .map_err(|err| RuntimeError::from(format!("Print error: {err}")))
    }

    fn execute_var_stmt(
        &mut self,
        name: &Token,
        initializer: Option<&Expr>,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        let value = if let Some(initializer) = initializer {
            self.evaluate(initializer, env)?
        } else {
            Cell::from(())
        };
        env.borrow_mut().define(name, value);
        Ok(())
    }

    fn execute_while_stmt(
        &mut self,
        condition: &Expr,
        body: &Stmt,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError> {
        while self.evaluate(condition, env)?.is_truthy() {
            self.execute(body, env)?
        }
        Ok(())
    }

    fn execute_assign_expr(
        &mut self,
        name: &Token,
        expr: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<Cell, RuntimeError> {
        let value = self.evaluate(expr, env)?;
        env.borrow_mut().assign(name, value.to_owned())?;
        Ok(value)
    }

    fn evaluate_literal(&self, literal: &Cell) -> Result<Cell, RuntimeError> {
        Ok(literal.to_owned())
    }

    fn evaluate_unary(
        &mut self,
        operator: &Token,
        right: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<Cell, RuntimeError> {
        let right = self.evaluate(right, env)?;
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
        env: &Arc<RefCell<Environment>>,
    ) -> Result<Cell, RuntimeError> {
        let left = self.evaluate(left, env)?;
        let right = self.evaluate(right, env)?;
        match operator.kind {
            TokenKind::Minus => {
                self.check_number_operands(operator, &left, &right)?;
                value::binary_operation(|a: f64, b: f64| a - b, left, operator, right)
            }
            TokenKind::Plus => {
                if left.is_number() && right.is_number() {
                    value::binary_operation(|a: f64, b: f64| a + b, left, operator, right)
                } else if left.is_string() && right.is_string() {
                    value::binary_operation(
                        |a: String, b: Arc<str>| Arc::from(a + &b),
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

    fn evaluate_logical(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<Cell, RuntimeError> {
        let left = self.evaluate(left, env)?;
        if operator.kind == TokenKind::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else if operator.kind == TokenKind::And && !left.is_truthy() {
            return Ok(left);
        }
        self.evaluate(right, env)
    }

    fn evaluate_ternary(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
        env: &Arc<RefCell<Environment>>,
    ) -> Result<Cell, RuntimeError> {
        let condition = self.evaluate(condition, env)?;
        if condition.is_truthy() {
            self.evaluate(then_expr, env)
        } else {
            self.evaluate(else_expr, env)
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
    use std::{io, sync::Arc};

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

    #[test]
    fn block_works() {
        assert_prints(
            r#"
            var a = "global a";
            var b = "global b";
            var c = "global c";
            {
                var a = "outer a";
                var b = "outer b";
                {
                    var a = "inner a";
                    print a;
                    print b;
                    print c;
                }
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
        "#,
            b"inner a\nouter b\nglobal c\nouter a\nouter b\nglobal c\nglobal a\nglobal b\nglobal c\n",
        );
    }

    #[test]
    fn logical_or_works() {
        assert_prints(
            r#"
            print "hi" or 2;
            print nil or "yes";
            print nil or false or 5 or 6;
        "#,
            b"hi\nyes\n5\n",
        )
    }

    #[test]
    fn logical_and_works() {
        assert_prints(
            r#"
            print "hi" and 2;
            print nil and "yes";
            print false and nil and 5 and 6;
            print 3 and 4 and 5 and 6;
        "#,
            b"2\nnil\nfalse\n6\n",
        )
    }

    #[test]
    fn if_stmt_works() {
        assert_prints(
            r#"
            if (true) {
                print "yes";
            } else {
                print "no";
            }
            if (0) {
                print "yes";
            } else {
                print "no";
            }
            if (nil) {
                print "yes";
            } else {
                print "no";
            }
            if (false) {
                print "yes";
            } else {
                print "no";
            }
        "#,
            b"yes\nyes\nno\nno\n",
        )
    }

    #[test]
    fn nested_if_stmt_works() {
        assert_prints(
            r#"
            if (true)
                if (true)
                    print "thenTrueTrue";
                else
                    print "elseTrueTrue";
            
            if (true)
                if (false)
                    print "thenTrueFalse";
                else
                    print "elseTrueFalse";

            if (false)
                if (true)
                    print "thenFalseTrue";
                else
                    print "elseFalseTrue";

            if (false)
                if (false)
                    print "thenFalseFalse";
                else
                    print "elseFalseFalse";
        "#,
            b"thenTrueTrue\nelseTrueFalse\n",
        )
    }

    #[test]
    fn while_stmt_works() {
        assert_prints(
            r#"
            var n = 5;
            var f = 1;
            while (n > 0) {
                f = f * n;
                n = n - 1;
            }
            print f;
        "#,
            b"120\n",
        );
    }

    #[test]
    fn for_stmt_works() {
        assert_prints(
            r#"
            var a = 0;
            var temp;

            for (var b = 1; a < 10000; b = temp + b) {
                print a;
                temp = a;
                a = b;
            }
        "#,
            b"0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n89\n144\n233\n377\n610\n987\n1597\n2584\n4181\n6765\n",
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
        let mut output = io::stdout();
        let mut interpreter = Interpreter::new_with_output(&error_reporter, &mut output);
        interpreter
            .evaluate_and_print(expr)
            .context("Evaluating error")
    }

    fn test_parse(source: &str, error_reporter: &ErrorReporter) -> Option<Box<[Box<Stmt>]>> {
        let scanner = Scanner::new(error_reporter);
        let tokens: Vec<_> = scanner.scan_tokens(source).collect();
        let mut parser = Parser::new(tokens, error_reporter);
        parser.parse()
    }
}
