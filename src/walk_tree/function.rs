use std::{fmt::Display, sync::Arc};

use super::{
    callable::{Callable, Context},
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
}

impl Function {
    pub fn init(
        name: Token,
        parameters: Arc<[Token]>,
        body: Arc<[Box<Stmt>]>,
    ) -> Arc<dyn Callable> {
        Arc::new(Self {
            name,
            parameters,
            body,
        })
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(&self, context: &mut dyn Context, arguments: &[Cell]) -> Result<Cell, RuntimeError> {
        let environment = Environment::new_with_enclosing(context.globals());
        for (i, parameter) in self.parameters.iter().enumerate() {
            environment
                .borrow_mut()
                .define(parameter.lexeme(), arguments[i].to_owned())
        }
        context.execute_block(&self.body, &environment)?;
        Ok(Cell::from(()))
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}
