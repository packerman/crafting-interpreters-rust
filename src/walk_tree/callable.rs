use std::fmt::Debug;

use super::{error::RuntimeError, value::Cell};

pub trait Context {}

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call(&self, context: &mut dyn Context, arguments: &[Cell]) -> Result<Cell, RuntimeError>;
}
