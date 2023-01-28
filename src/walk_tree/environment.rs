use std::{cell::RefCell, collections::HashMap, sync::Arc};

use super::{error::RuntimeError, token::Token, value::Cell};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Arc<RefCell<Environment>>>,
    values: HashMap<Arc<str>, Cell>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_with_enclosing(enclosing: Arc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &Token, value: Cell) {
        self.values.insert(name.lexeme.to_owned(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Cell, RuntimeError> {
        if let Some(cell) = self.values.get(&name.lexeme) {
            Ok(cell.to_owned())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Cell) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_owned(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }
}
