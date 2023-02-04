use std::{fmt::Display, sync::Arc};

use super::{error::RuntimeError, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell(Option<Arc<Value>>);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(Arc<str>),
}

impl From<Value> for Cell {
    fn from(value: Value) -> Self {
        Self(Some(Arc::new(value)))
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

impl From<Arc<str>> for Cell {
    fn from(value: Arc<str>) -> Self {
        Self::from(Value::String(value))
    }
}

impl TryFrom<Cell> for Arc<str> {
    type Error = String;

    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        if let Some(Value::String(v)) = value.0.as_deref() {
            Ok(Arc::clone(v))
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

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_deref() {
            Some(Value::Boolean(value)) => write!(f, "{value}"),
            None => write!(f, "nil"),
            Some(Value::Number(value)) => write!(f, "{value}"),
            Some(Value::String(value)) => write!(f, "{value}"),
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
