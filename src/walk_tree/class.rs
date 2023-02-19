use std::{
    fmt::Display,
    rc::{Rc, Weak},
};

use super::{
    callable::{Callable, ExecutionContext},
    error::RuntimeError,
    value::Cell,
};

#[derive(Debug, Clone)]
pub struct Class {
    name: Rc<str>,
    me: Weak<Self>,
}

impl Class {
    pub fn new(name: Rc<str>) -> Rc<Self> {
        Rc::new_cyclic(|me| Self {
            name,
            me: me.clone(),
        })
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _context: &mut dyn ExecutionContext,
        _arguments: &[Cell],
    ) -> Result<Cell, RuntimeError> {
        Ok(Cell::from(Instance::new(
            self.me.upgrade().expect("Reference exists"),
        )))
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    class: Rc<Class>,
}

impl Instance {
    fn new(class: Rc<Class>) -> Self {
        Self { class }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
