use std::fmt::Display;

use super::{error::RuntimeError, token::Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Value(Option<TypedValue>);

#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    Boolean(bool),
    Number(f64),
    String(String),
}

impl From<TypedValue> for Value {
    fn from(value: TypedValue) -> Self {
        Self(Some(value))
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::from(TypedValue::Boolean(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::from(TypedValue::Number(v))
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::from(TypedValue::String(String::from(v)))
    }
}

impl TryFrom<Value> for f64 {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Some(TypedValue::Number(v)) = value.0 {
            Ok(v)
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::from(TypedValue::String(v))
    }
}

impl TryFrom<Value> for String {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Some(TypedValue::String(v)) = value.0 {
            Ok(v)
        } else {
            Err(String::from("Expect number."))
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(TypedValue::Boolean(value)) => write!(f, "{}", value),
            None => write!(f, "nil"),
            Some(TypedValue::Number(value)) => write!(f, "{}", value),
            Some(TypedValue::String(value)) => write!(f, "{}", value),
        }
    }
}

impl Value {
    pub fn try_into_number(self) -> Result<f64, Self> {
        if let Some(TypedValue::Number(v)) = self.0 {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self.0 {
            None => false,
            Some(TypedValue::Boolean(value)) => value.to_owned(),
            _ => true,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.0, Some(TypedValue::Number(..)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.0, Some(TypedValue::String(..)))
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
