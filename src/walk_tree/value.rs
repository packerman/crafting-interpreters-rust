use std::fmt::Display;

use super::{error::RuntimeError, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl TryFrom<Value> for f64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Number(v) = value {
            Ok(v)
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl TryFrom<Value> for String {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(v) = value {
            Ok(v)
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "nil"),
            Value::Number(value) => write!(f, "{}", value),
            Value::String(value) => write!(f, "{}", value),
        }
    }
}

impl Value {
    pub fn try_into_number(self) -> Result<f64, Self> {
        if let Self::Number(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Nil => false,
            Self::Boolean(value) => value.to_owned(),
            _ => true,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }
}

pub fn unary_operation<T>(
    op: fn(T) -> T,
    operator: &Token,
    right: Value,
) -> Result<Value, RuntimeError>
where
    T: TryFrom<Value, Error = String>,
    Value: From<T>,
    <T as TryFrom<Value>>::Error: std::fmt::Debug,
{
    let value = Value::from(op(right
        .try_into()
        .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?));
    Ok(value)
}

pub fn binary_operation<T>(
    operation: fn(T, T) -> T,
    left: Value,
    operator: &Token,
    right: Value,
) -> Result<Value, RuntimeError>
where
    T: TryFrom<Value, Error = String>,
    Value: From<T>,
    <T as TryFrom<Value>>::Error: std::fmt::Debug,
{
    let value = Value::from(operation(
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
    left: Value,
    operator: &Token,
    right: Value,
) -> Result<Value, RuntimeError>
where
    T: TryFrom<Value, Error = String>,
    <T as TryFrom<Value>>::Error: std::fmt::Debug,
{
    let value = Value::from(relation(
        left.try_into()
            .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?,
        right
            .try_into()
            .map_err(|message: String| RuntimeError::new(operator.to_owned(), &message))?,
    ));
    Ok(value)
}
