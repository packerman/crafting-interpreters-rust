use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{error::RuntimeError, token::Token, value::Cell};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<Rc<str>, Cell>,
    me: Weak<RefCell<Self>>,
}

impl Environment {
    pub fn new_global() -> Rc<RefCell<Self>> {
        Self::new(None)
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Self::new(Some(enclosing))
    }

    fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|me| {
            RefCell::new(Self {
                enclosing,
                values: HashMap::new(),
                me: me.clone(),
            })
        })
    }

    pub fn define(&mut self, name: Rc<str>, value: Cell) {
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

    pub fn get_at(&self, distance: usize, name: &Rc<str>) -> Cell {
        self.ancestor(distance).borrow().values[name].to_owned()
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

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = self.me.upgrade().expect("Reference exists");
        for _ in 0..distance {
            let enclosing = Rc::clone(
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
