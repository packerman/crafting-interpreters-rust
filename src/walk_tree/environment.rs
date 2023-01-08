use std::{collections::HashMap, rc::Rc};

use super::{error::RuntimeError, token::Token, value::Value};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Rc<Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Rc<Value>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<&Rc<Value>, RuntimeError> {
        self.values.get(&name.lexeme).ok_or_else(|| {
            RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme),
            )
        })
    }
}
