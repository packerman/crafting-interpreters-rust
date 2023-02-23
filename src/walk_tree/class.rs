use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    rc::{Rc, Weak},
};

use super::{
    callable::{Callable, ExecutionContext},
    error::RuntimeError,
    token::Token,
    value::Cell,
};

#[derive(Debug, Clone)]
pub struct Class {
    name: Rc<str>,
    methods: HashMap<Rc<str>, Rc<dyn Callable>>,
    me: Weak<Self>,
}

impl Class {
    pub fn new(name: Rc<str>, methods: HashMap<Rc<str>, Rc<dyn Callable>>) -> Rc<Self> {
        Rc::new_cyclic(|me| Self {
            name,
            methods,
            me: me.clone(),
        })
    }

    pub fn find_method(&self, name: &str) -> Option<&Rc<dyn Callable>> {
        self.methods.get(name)
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
    fields: HashMap<Rc<str>, Cell>,
}

impl Instance {
    fn new(class: Rc<Class>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            class,
            fields: HashMap::new(),
        }))
    }

    pub fn get(&self, name: &Token) -> Result<Cell, RuntimeError> {
        if let Some(value) = self.fields.get(name.lexeme()) {
            Ok(value.to_owned())
        } else if let Some(method) = self.class.find_method(name.lexeme()) {
            Ok(Cell::from(Rc::clone(method)))
        } else {
            Err(RuntimeError::new(
                name.to_owned(),
                &format!("Undefined property '{}'.", name.lexeme()),
            ))
        }
    }

    pub fn set(&mut self, name: &Token, value: Cell) {
        self.fields.insert(Rc::clone(name.lexeme()), value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
