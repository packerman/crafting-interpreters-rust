use std::{cell::RefCell, fmt::Debug, ptr, sync::Arc};

use super::{
    control_flow::ControlFlow, environment::Environment, error::RuntimeError, stmt::Stmt,
    value::Cell,
};

pub trait ExecutionContext {
    fn globals(&self) -> Arc<RefCell<Environment>>;

    fn execute_block(
        &mut self,
        block: &[Box<Stmt>],
        env: &Arc<RefCell<Environment>>,
    ) -> Result<(), ControlFlow>;
}

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call(
        &self,
        context: &mut dyn ExecutionContext,
        arguments: &[Cell],
    ) -> Result<Cell, RuntimeError>;
}

#[allow(clippy::vtable_address_comparisons)]
pub fn ptr_eq(left: &dyn Callable, right: &dyn Callable) -> bool {
    ptr::eq(left, right)
}
