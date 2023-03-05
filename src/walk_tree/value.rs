use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{
    callable::{self, Callable},
    class::{Class, Instance},
    error::RuntimeError,
    function::Function,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell(Option<Value>);

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(Rc<str>),
    Function(Rc<dyn Callable>),
    Class(Rc<Class>),
    Instance(Rc<RefCell<Instance>>),
}

impl Value {
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class(..))
    }

    pub fn as_class(&self) -> Option<&Rc<Class>> {
        if let Self::Class(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Function(left), Self::Function(right)) => {
                callable::ptr_eq(left.as_ref(), right.as_ref())
            }
            _ => false,
        }
    }
}

impl From<Value> for Cell {
    fn from(value: Value) -> Self {
        Self(Some(value))
    }
}

impl From<bool> for Cell {
    fn from(v: bool) -> Self {
        Self::from(Value::Boolean(v))
    }
}

impl From<f64> for Cell {
    fn from(v: f64) -> Self {
        Self::from(Value::Number(v))
    }
}

impl TryFrom<Cell> for f64 {
    type Error = String;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::Number(v)) = value.0 {
            Ok(v)
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl From<Rc<str>> for Cell {
    fn from(value: Rc<str>) -> Self {
        Self::from(Value::String(value))
    }
}

impl TryFrom<Cell> for Rc<str> {
    type Error = String;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::String(v)) = value.0 {
            Ok(v)
        } else {
            Err(String::from("Expect string."))
        }
    }
}

impl TryFrom<Cell> for String {
    type Error = String;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::String(v)) = value.0 {
            Ok(v.to_string())
        } else {
            Err(String::from("Expect string."))
        }
    }
}

impl From<()> for Cell {
    fn from(_value: ()) -> Self {
        Self(None)
    }
}

impl From<Rc<dyn Callable>> for Cell {
    fn from(value: Rc<dyn Callable>) -> Self {
        Cell::from(Value::Function(value))
    }
}

impl From<Rc<Function>> for Cell {
    fn from(value: Rc<Function>) -> Self {
        Cell::from(Value::Function(value))
    }
}

impl TryFrom<Cell> for Rc<dyn Callable> {
    type Error = RuntimeError;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::Function(value)) = value.0 {
            Ok(value)
        } else if let Some(Value::Class(class)) = value.0 {
            Ok(class)
        } else {
            Err(RuntimeError::from(String::from(
                "Can only call functions and classes.",
            )))
        }
    }
}

impl TryFrom<Cell> for Rc<RefCell<Instance>> {
    type Error = RuntimeError;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::Instance(value)) = value.0 {
            Ok(value)
        } else {
            Err(RuntimeError::from(String::from(
                "Only instances have properties.",
            )))
        }
    }
}

impl From<Rc<Class>> for Cell {
    fn from(value: Rc<Class>) -> Self {
        Cell::from(Value::Class(value))
    }
}

impl From<Rc<RefCell<Instance>>> for Cell {
    fn from(value: Rc<RefCell<Instance>>) -> Self {
        Cell::from(Value::Instance(value))
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(Value::Boolean(value)) => write!(f, "{value}"),
            None => write!(f, "nil"),
            Some(Value::Number(value)) => write!(f, "{value}"),
            Some(Value::String(value)) => write!(f, "{value}"),
            Some(Value::Function(value)) => write!(f, "<function@{value:p}>"),
            Some(Value::Class(value)) => write!(f, "{value}"),
            Some(Value::Instance(value)) => write!(f, "{}", value.borrow()),
        }
    }
}

impl Cell {
    pub fn is_truthy(&self) -> bool {
        match self.0 {
            None => false,
            Some(Value::Boolean(value)) => value,
            _ => true,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.0, Some(Value::Number(..)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.0, Some(Value::String(..)))
    }

    pub fn is_class(&self) -> bool {
        self.0.as_ref().map_or(false, |value| value.is_class())
    }

    pub fn as_class(&self) -> Option<&Rc<Class>> {
        self.0.as_ref().and_then(|value| value.as_class())
    }
}

pub fn unary_operation<T, R>(
    op: fn(T) -> R,
    operator: &Token,
    right: Cell,
) -> Result<Cell, RuntimeError>
where
    T: TryFrom<Cell, Error = String>,
    Cell: From<R>,
    <T as TryFrom<Cell>>::Error: std::fmt::Debug,
{
    let value = Cell::from(op(right
        .try_into()
        .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?));
    Ok(value)
}

pub fn binary_operation<T, R, S>(
    operation: fn(T, R) -> S,
    left: Cell,
    operator: &Token,
    right: Cell,
) -> Result<Cell, RuntimeError>
where
    T: TryFrom<Cell, Error = String>,
    R: TryFrom<Cell, Error = String>,
    Cell: From<S>,
    <T as TryFrom<Cell>>::Error: std::fmt::Debug,
    <R as TryFrom<Cell>>::Error: std::fmt::Debug,
{
    let value = Cell::from(operation(
        left.try_into()
            .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?,
        right
            .try_into()
            .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?,
    ));
    Ok(value)
}

pub fn binary_relation<T>(
    relation: fn(T, T) -> bool,
    left: Cell,
    operator: &Token,
    right: Cell,
) -> Result<Cell, RuntimeError>
where
    T: TryFrom<Cell, Error = String>,
    <T as TryFrom<Cell>>::Error: std::fmt::Debug,
{
    let value = Cell::from(relation(
        left.try_into()
            .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?,
        right
            .try_into()
            .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?,
    ));
    Ok(value)
}
