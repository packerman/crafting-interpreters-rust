use std::{collections::HashMap, rc::Rc};

use super::{
    error::ErrorReporter,
    expr::{Expr, Function},
    stmt::Stmt,
    token::Token,
};

pub trait Resolve {
    fn resolve(&mut self, expr: *const Expr, depth: usize);
}

pub struct Resolver<'a> {
    interpreter: &'a mut dyn Resolve,
    error_reporter: &'a ErrorReporter,
    scopes: Vec<HashMap<Rc<str>, bool>>,
    current_function: Option<FunctionType>,
    current_class: Option<ClassType>,
    this_keyword: Rc<str>,
    super_keyword: Rc<str>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut dyn Resolve, error_reporter: &'a ErrorReporter) -> Self {
        Self {
            interpreter,
            error_reporter,
            scopes: Vec::new(),
            current_function: None,
            current_class: None,
            this_keyword: Rc::from("this"),
            super_keyword: Rc::from("super"),
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
            Stmt::Class {
                name,
                superclass,
                methods,
            } => self.resolve_class_stmt(name, superclass.as_deref(), methods),
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
            Expr::Function(function) => self.resolve_function_expr(function),
            Expr::Get { object, name } => self.resolve_get_expr(object, name),
            Expr::Set {
                object,
                name,
                value,
            } => self.resolve_set_expr(object, name, value),
            Expr::This { keyword } => self.resolve_this_expr(expr, keyword),
            Expr::Super { keyword, method } => self.resolve_super_expr(expr, keyword, method),
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

    fn resolve_function_expr(&mut self, function: &Function) {
        if let Some(name) = function.name() {
            self.declare(name);
            self.define(name)
        }
        self.resolve_function(function, FunctionType::Function);
    }

    fn resolve_function(&mut self, function: &Function, function_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = Some(function_type);

        self.begin_scope();
        for param in function.parameters().iter() {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(function.body());
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
            if self.current_function == Some(FunctionType::Initializer) {
                self.error_reporter
                    .token_error(keyword, "Can't return a value from an initializer.")
            }
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

    fn resolve_class_stmt(
        &mut self,
        name: &Token,
        superclass: Option<&Expr>,
        methods: &[Function],
    ) {
        let enclosing_class = self.current_class;
        self.current_class = Some(ClassType::Class);

        self.declare(name);
        self.define(name);

        if let Some(superclass) = superclass {
            let superclass_name = superclass.as_variable().expect("Expect identifier.");
            if name.lexeme() == superclass_name.lexeme() {
                self.error_reporter
                    .token_error(superclass_name, "A class can't inherit from itself.");
            }

            self.resolve_expr(superclass);

            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert(Rc::clone(&self.super_keyword), true);
        }

        self.begin_scope();
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(Rc::clone(&self.this_keyword), true);
        }
        for method in methods {
            let declaration = if method
                .name()
                .map_or(false, |name| name.lexeme().as_ref() == "init")
            {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(method, declaration);
        }
        self.end_scope();
        if superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;
    }

    fn resolve_get_expr(&mut self, object: &Expr, _name: &Token) {
        self.resolve_expr(object)
    }

    fn resolve_set_expr(&mut self, object: &Expr, _name: &Token, value: &Expr) {
        self.resolve_expr(value);
        self.resolve_expr(object);
    }

    fn resolve_this_expr(&mut self, expr: &Expr, keyword: &Token) {
        if self.current_class.is_none() {
            self.error_reporter
                .token_error(keyword, "Can't use 'this' outside of a class.");
            return;
        }

        self.resolve_local(expr, keyword)
    }

    fn resolve_super_expr(&mut self, expr: &Expr, keyword: &Token, _method: &Token) {
        self.resolve_local(expr, keyword)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionType {
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone, Copy)]
enum ClassType {
    Class,
}
