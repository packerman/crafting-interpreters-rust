use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Weak},
};

use super::{error::RuntimeError, token::Token, value::Cell};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Arc<RefCell<Environment>>>,
    values: HashMap<Arc<str>, Cell>,
    me: Weak<RefCell<Self>>,
}

impl Environment {
    pub fn new_global() -> Arc<RefCell<Self>> {
        Self::new(None)
    }

    pub fn new_with_enclosing(enclosing: Arc<RefCell<Environment>>) -> Arc<RefCell<Self>> {
        Self::new(Some(enclosing))
    }

    fn new(enclosing: Option<Arc<RefCell<Environment>>>) -> Arc<RefCell<Self>> {
        Arc::new_cyclic(|me| {
            RefCell::new(Self {
                enclosing,
                values: HashMap::new(),
                me: me.clone(),
            })
        })
    }

    pub fn define(&mut self, name: Arc<str>, value: Cell) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Cell, RuntimeError> {
        if let Some(cell) = self.values.get(name.lexeme()) {
            Ok(cell.to_owned())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme()),
            ))
        }
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Cell {
        self.ancestor(distance).borrow().values[name.lexeme()].to_owned()
    }

    pub fn assign(&mut self, name: &Token, value: Cell) -> Result<(), RuntimeError> {
        if self.values.contains_key(name.lexeme()) {
            self.values.insert(name.lexeme().to_owned(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::new(
                name.to_owned(),
                &format!("Undefined variable '{}'.", name.lexeme()),
            ))
        }
    }

    pub fn assing_at(&self, distance: usize, name: &Token, value: Cell) {
        self.ancestor(distance)
            .borrow_mut()
            .values
            .insert(name.lexeme().to_owned(), value);
    }

    fn ancestor(&self, distance: usize) -> Arc<RefCell<Environment>> {
        let mut environment = self.me.upgrade().expect("Reference exists");
        for _ in 0..distance {
            let enclosing = Arc::clone(
                environment
                    .borrow()
                    .enclosing
                    .as_ref()
                    .expect("Environment exists"),
            );
            environment = enclosing;
        }
        environment
    }
}
