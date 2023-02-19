use std::{collections::HashMap, rc::Rc};

use super::{error::ErrorReporter, expr::Expr, stmt::Stmt, token::Token};

pub trait Resolve {
    fn resolve(&mut self, expr: *const Expr, depth: usize);
}

pub struct Resolver<'a> {
    interpreter: &'a mut dyn Resolve,
    error_reporter: &'a ErrorReporter,
    scopes: Vec<HashMap<Rc<str>, bool>>,
    current_function: Option<FunctionType>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut dyn Resolve, error_reporter: &'a ErrorReporter) -> Self {
        Self {
            interpreter,
            error_reporter,
            scopes: Vec::new(),
            current_function: None,
        }
    }

    pub fn resolve(&mut self, stmts: &[Box<Stmt>]) {
        self.resolve_stmts(stmts)
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => self.resolve_block_stmt(stmts),
            Stmt::Expr(expression) => self.resolve_expression_stmt(expression),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.resolve_if_stmt(condition, then_branch, else_branch.as_deref()),
            Stmt::Return {
                keyword,
                expr: value,
            } => self.resolve_return_stmt(keyword, value.as_deref()),
            Stmt::While { condition, body } => self.resolve_while_stmt(condition, body),
            Stmt::VarDeclaration { name, initializer } => {
                self.resolve_var_stmt(name, initializer.as_deref())
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, right, .. } => self.resolve_binary_expr(left, right),
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => self.resolve_call_expr(callee, arguments),
            Expr::Unary {
                operator: _,
                operand: right,
            } => self.resolve_unary_expr(right),
            Expr::Literal(_) => {}
            Expr::Grouping(expression) => self.resolve_grouping_expr(expression),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => self.resolve_ternary_expr(condition, then_expr, else_expr),
            Expr::Variable(name) => self.resolve_variable_expr(expr, name),
            Expr::Assignment { name, value } => self.resolve_assign_expr(expr, name, value),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => self.resolve_logical_expression(left, right),
            Expr::Function {
                name,
                parameters: params,
                body,
            } => self.resolve_function_expr(name.as_ref(), params, body),
        }
    }

    fn resolve_block_stmt(&mut self, stmts: &[Box<Stmt>]) {
        self.begin_scope();
        self.resolve_stmts(stmts);
        self.end_scope();
    }

    fn resolve_stmts(&mut self, stmts: &[Box<Stmt>]) {
        for stmt in stmts {
            self.resolve_stmt(stmt)
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) {
        self.declare(name);
        if let Some(initializer) = initializer {
            self.resolve_expr(initializer)
        }
        self.define(name)
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.lexeme()) {
                self.error_reporter
                    .token_error(name, "Already a variable with this name in this scope.");
            }
            scope.insert(Rc::clone(name.lexeme()), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(Rc::clone(name.lexeme()), true);
        }
    }

    fn resolve_variable_expr(&mut self, expr: &Expr, name: &Token) {
        if let Some(scope) = self.scopes.last() {
            if scope.get(name.lexeme()).map_or(false, |defined| !defined) {
                self.error_reporter
                    .token_error(name, "Can't read local variable in its own initializer.")
            }
        }
        self.resolve_local(expr, name)
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name.lexeme()) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_assign_expr(&mut self, expr: &Expr, name: &Token, value: &Expr) {
        self.resolve_expr(value);
        self.resolve_local(expr, name);
    }

    fn resolve_function_expr(
        &mut self,
        name: Option<&Token>,
        params: &[Token],
        body: &[Box<Stmt>],
    ) {
        if let Some(name) = name {
            self.declare(name);
            self.define(name)
        }
        self.resolve_function(params, body, FunctionType::Function);
    }

    fn resolve_function(
        &mut self,
        params: &[Token],
        body: &[Box<Stmt>],
        function_type: FunctionType,
    ) {
        let enclosing_function = self.current_function;
        self.current_function = Some(function_type);

        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn resolve_expression_stmt(&mut self, expression: &Expr) {
        self.resolve_expr(expression)
    }

    fn resolve_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch)
        }
    }

    fn resolve_return_stmt(&mut self, keyword: &Token, value: Option<&Expr>) {
        if self.current_function.is_none() {
            self.error_reporter
                .token_error(keyword, "Can't return from top-level code.")
        }

        if let Some(value) = value {
            self.resolve_expr(value)
        }
    }

    fn resolve_while_stmt(&mut self, condition: &Expr, body: &Stmt) {
        self.resolve_expr(condition);
        self.resolve_stmt(body)
    }

    fn resolve_binary_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right)
    }

    fn resolve_call_expr(&mut self, callee: &Expr, arguments: &[Box<Expr>]) {
        self.resolve_expr(callee);
        for argument in arguments {
            self.resolve_expr(argument)
        }
    }

    fn resolve_grouping_expr(&mut self, expression: &Expr) {
        self.resolve_expr(expression)
    }

    fn resolve_logical_expression(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right)
    }

    fn resolve_unary_expr(&mut self, right: &Expr) {
        self.resolve_expr(right)
    }

    fn resolve_ternary_expr(&mut self, condition: &Expr, then_expr: &Expr, else_expr: &Expr) {
        self.resolve_expr(condition);
        self.resolve_expr(then_expr);
        self.resolve_expr(else_expr)
    }
}

#[derive(Debug, Clone, Copy)]
enum FunctionType {
    Function,
}
