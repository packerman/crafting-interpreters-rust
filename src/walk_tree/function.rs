use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{
    callable::{Callable, ExecutionContext},
    class::Instance,
    control_flow::ControlFlow,
    environment::Environment,
    error::RuntimeError,
    expr,
    stmt::Stmt,
    token::Token,
    value::Cell,
};

#[derive(Debug)]
pub struct Function {
    name: Option<Token>,
    parameters: Rc<[Token]>,
    body: Rc<[Box<Stmt>]>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
    this: Rc<str>,
}

impl Function {
    pub fn new(
        function: &expr::Function,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Rc<Self> {
        Rc::new(Self {
            name: function.name().cloned(),
            parameters: Rc::clone(function.parameters()),
            body: Rc::clone(function.body()),
            closure,
            is_initializer,
            this: Rc::from("this"),
        })
    }

    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Rc<Self> {
        let environment = Environment::new_with_enclosing(Rc::clone(&self.closure));
        environment
            .borrow_mut()
            .define(Rc::clone(&self.this), Cell::from(instance));
        Rc::new(Function {
            name: self.name.clone(),
            parameters: Rc::clone(&self.parameters),
            body: Rc::clone(&self.body),
            closure: environment,
            is_initializer: self.is_initializer,
            this: Rc::clone(&self.this),
        })
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(
        &self,
        context: &mut dyn ExecutionContext,
        arguments: &[Cell],
    ) -> Result<Cell, RuntimeError> {
        let environment = Environment::new_with_enclosing(Rc::clone(&self.closure));
        for (i, parameter) in self.parameters.iter().enumerate() {
            environment
                .borrow_mut()
                .define(Rc::clone(parameter.lexeme()), arguments[i].to_owned())
        }
        let result = context.execute_block(&self.body, &environment);
        match result {
            Err(ControlFlow::Return(value)) => Ok(if self.is_initializer {
                self.closure.borrow().get_at(0, &self.this)
            } else {
                value
            }),
            Err(ControlFlow::RuntimeError(runtime_error)) => Err(runtime_error),
            _ => Ok(if self.is_initializer {
                self.closure.borrow().get_at(0, &self.this)
            } else {
                Cell::from(())
            }),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "<fn {}>", name.lexeme())
        } else {
            write!(f, "<anonymous fn>")
        }
    }
}
