use std::{error::Error, fmt::Display};

use super::{error::RuntimeError, value::Cell};

#[derive(Debug)]
pub enum ControlFlow {
    RuntimeError(RuntimeError),
    Return(Cell),
}

impl From<RuntimeError> for ControlFlow {
    fn from(value: RuntimeError) -> Self {
        ControlFlow::RuntimeError(value)
    }
}

impl From<Cell> for ControlFlow {
    fn from(value: Cell) -> Self {
        ControlFlow::Return(value)
    }
}

impl Display for ControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFlow::RuntimeError(runtime_error) => write!(f, "{runtime_error}"),
            ControlFlow::Return(value) => write!(f, "{value}"),
        }
    }
}

impl Error for ControlFlow {}
