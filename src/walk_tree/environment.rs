use std::{collections::HashMap, sync::Arc};

use super::{error::RuntimeError, token::Token, value::Cell};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<Arc<str>, Cell>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &Token, value: Cell) {
        self.values.insert(name.lexeme.to_owned(), value);
    }

    pub fn get(&self, name: &Token) -> Result<&Cell, RuntimeError> {
        self.values.get(&name.lexeme).ok_or_else(|| {
            RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme),
            )
        })
    }

    pub fn assign(&mut self, name: &Token, value: Cell) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_owned(), value);
            Ok(())
        } else {
            Err(RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }
}
