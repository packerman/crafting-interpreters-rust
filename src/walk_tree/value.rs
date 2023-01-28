use std::{fmt::Display, sync::Arc};

use super::{error::RuntimeError, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell(Option<Value>);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(Arc<str>),
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

impl From<&str> for Cell {
    fn from(value: &str) -> Self {
        Self::from(Value::String(Arc::from(value)))
    }
}

impl From<Arc<str>> for Cell {
    fn from(value: Arc<str>) -> Self {
        Self::from(Value::String(value))
    }
}

impl From<()> for Cell {
    fn from(_value: ()) -> Self {
        Self(None)
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

impl From<String> for Cell {
    fn from(v: String) -> Self {
        Self::from(Value::String(Arc::from(v)))
    }
}

impl TryFrom<Cell> for String {
    type Error = String;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::String(v)) = value.0 {
            Ok(v.to_string())
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_ref() {
            Some(Value::Boolean(value)) => write!(f, "{}", value),
            None => write!(f, "nil"),
            Some(Value::Number(value)) => write!(f, "{}", value),
            Some(Value::String(value)) => write!(f, "{}", value),
        }
    }
}

impl Cell {
    pub fn is_truthy(&self) -> bool {
        match self.0.as_ref() {
            None => false,
            Some(Value::Boolean(value)) => value.to_owned(),
            _ => true,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.0.as_ref(), Some(Value::Number(..)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.0.as_ref(), Some(Value::String(..)))
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

pub fn binary_operation<T, R>(
    operation: fn(T, T) -> R,
    left: Cell,
    operator: &Token,
    right: Cell,
) -> Result<Cell, RuntimeError>
where
    T: TryFrom<Cell, Error = String>,
    Cell: From<R>,
    <T as TryFrom<Cell>>::Error: std::fmt::Debug,
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
