use std::{cell::RefCell, fmt::Debug, ptr, sync::Arc};

use super::{environment::Environment, error::RuntimeError, stmt::Stmt, value::Cell};

pub trait Context {
    fn globals(&self) -> Arc<RefCell<Environment>>;

    fn execute_block(
        &mut self,
        block: &[Box<Stmt>],
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), RuntimeError>;
}

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call(&self, context: &mut dyn Context, arguments: &[Cell]) -> Result<Cell, RuntimeError>;
}

#[allow(clippy::vtable_address_comparisons)]
pub fn ptr_eq(left: &dyn Callable, right: &dyn Callable) -> bool {
    ptr::eq(left, right)
}
