use std::{fmt::Debug, ptr};

use super::{error::RuntimeError, value::Cell};

pub trait Context {}

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call(&self, context: &mut dyn Context, arguments: &[Cell]) -> Result<Cell, RuntimeError>;
}

#[allow(clippy::vtable_address_comparisons)]
pub fn ptr_eq(left: &dyn Callable, right: &dyn Callable) -> bool {
    ptr::eq(left, right)
}
