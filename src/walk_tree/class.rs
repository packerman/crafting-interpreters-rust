use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    rc::{Rc, Weak},
};

use super::{
    callable::{Callable, ExecutionContext},
    error::RuntimeError,
    function::Function,
    token::Token,
    value::Cell,
};

#[derive(Debug, Clone)]
pub struct Class {
    name: Rc<str>,
    superclass: Option<Rc<Class>>,
    methods: HashMap<Rc<str>, Rc<Function>>,
    me: Weak<Self>,
}

impl Class {
    pub fn new(
        name: Rc<str>,
        superclass: Option<Rc<Class>>,
        methods: HashMap<Rc<str>, Rc<Function>>,
    ) -> Rc<Self> {
        Rc::new_cyclic(|me| Self {
            name,
            superclass,
            methods,
            me: me.clone(),
        })
    }

    pub fn find_method(&self, name: &str) -> Option<&Rc<Function>> {
        self.methods.get(name).or_else(|| {
            self.superclass
                .as_ref()
                .and_then(|superclass| superclass.find_method(name))
        })
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(
        &self,
        context: &mut dyn ExecutionContext,
        arguments: &[Cell],
    ) -> Result<Cell, RuntimeError> {
        let instance = Instance::new(self.me.upgrade().expect("Reference exists"));
        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(Rc::clone(&instance))
                .call(context, arguments)?;
        }
        Ok(Cell::from(instance))
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
    me: Weak<RefCell<Self>>,
}

impl Instance {
    fn new(class: Rc<Class>) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|me| {
            RefCell::new(Self {
                class,
                fields: HashMap::new(),
                me: me.clone(),
            })
        })
    }

    pub fn get(&self, name: &Token) -> Result<Cell, RuntimeError> {
        if let Some(value) = self.fields.get(name.lexeme()) {
            Ok(value.to_owned())
        } else if let Some(method) = self.class.find_method(name.lexeme()) {
            let method = Rc::clone(method).bind(self.me());
            Ok(Cell::from(method))
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

    pub fn me(&self) -> Rc<RefCell<Self>> {
        self.me.upgrade().unwrap()
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
