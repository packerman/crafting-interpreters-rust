use std::{fmt::Display, rc::Rc};

use super::{
    callable::{self, Callable},
    class::{Class, Instance},
    error::RuntimeError,
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell(Option<Rc<Value>>);

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(Rc<str>),
    Callable(Rc<dyn Callable>),
    Class(Rc<Class>),
    Instance(Instance),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Callable(left), Self::Callable(right)) => {
                callable::ptr_eq(left.as_ref(), right.as_ref())
            }
            _ => false,
        }
    }
}

impl From<Value> for Cell {
    fn from(value: Value) -> Self {
        Self(Some(Rc::new(value)))
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
        if let Some(Value::Number(v)) = value.0.as_deref() {
            Ok(*v)
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
        if let Some(Value::String(v)) = value.0.as_deref() {
            Ok(Rc::clone(v))
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl TryFrom<Cell> for String {
    type Error = String;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::String(v)) = value.0.as_deref() {
            Ok(v.to_string())
        } else {
            Err(String::from("Expect number."))
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
        Cell::from(Value::Callable(value))
    }
}

impl TryFrom<Cell> for Rc<dyn Callable> {
    type Error = RuntimeError;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::Callable(value)) = value.0.as_deref() {
            Ok(Rc::clone(value))
        } else if let Some(Value::Class(class)) = value.0.as_deref() {
            Ok(callable::as_callable(Rc::clone(class)))
        } else {
            Err(RuntimeError::from(String::from(
                "Can only call functions and classes.",
            )))
        }
    }
}

impl From<Rc<Class>> for Cell {
    fn from(value: Rc<Class>) -> Self {
        Cell::from(Value::Class(value))
    }
}

impl From<Instance> for Cell {
    fn from(value: Instance) -> Self {
        Cell::from(Value::Instance(value))
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_deref() {
            Some(Value::Boolean(value)) => write!(f, "{value}"),
            None => write!(f, "nil"),
            Some(Value::Number(value)) => write!(f, "{value}"),
            Some(Value::String(value)) => write!(f, "{value}"),
            Some(Value::Callable(value)) => write!(f, "<function@{value:p}>"),
            Some(Value::Class(value)) => write!(f, "{value}"),
            Some(Value::Instance(value)) => write!(f, "{value}"),
        }
    }
}

impl Cell {
    pub fn is_truthy(&self) -> bool {
        match self.0.as_deref() {
            None => false,
            Some(Value::Boolean(value)) => value.to_owned(),
            _ => true,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.0.as_deref(), Some(Value::Number(..)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.0.as_deref(), Some(Value::String(..)))
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
