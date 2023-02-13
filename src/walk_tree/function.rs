use std::{cell::RefCell, fmt::Display, sync::Arc};

use super::{
    callable::{Callable, ExecutionContext},
    control_flow::ControlFlow,
    environment::Environment,
    error::RuntimeError,
    stmt::Stmt,
    token::Token,
    value::Cell,
};

#[derive(Debug)]
pub struct Function {
    name: Token,
    parameters: Arc<[Token]>,
    body: Arc<[Box<Stmt>]>,
    closure: Arc<RefCell<Environment>>,
}

impl Function {
    pub fn init(
        name: Token,
        parameters: Arc<[Token]>,
        body: Arc<[Box<Stmt>]>,
        closure: Arc<RefCell<Environment>>,
    ) -> Arc<dyn Callable> {
        Arc::new(Self {
            name,
            parameters,
            body,
            closure,
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
        let environment = Environment::new_with_enclosing(Arc::clone(&self.closure));
        for (i, parameter) in self.parameters.iter().enumerate() {
            environment
                .borrow_mut()
                .define(parameter.lexeme(), arguments[i].to_owned())
        }
        let result = context.execute_block(&self.body, &environment);
        match result {
            Err(ControlFlow::Return(value)) => Ok(value),
            Err(ControlFlow::RuntimeError(runtime_error)) => Err(runtime_error),
            _ => Ok(Cell::from(())),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}
