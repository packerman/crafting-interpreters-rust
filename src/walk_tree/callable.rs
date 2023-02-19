use std::{cell::RefCell, fmt::Debug, io::Write, ptr, rc::Rc};

use super::{
    control_flow::ControlFlow, environment::Environment, error::RuntimeError, stmt::Stmt,
    value::Cell,
};

pub trait ExecutionContext {
    fn globals(&self) -> Rc<RefCell<Environment>>;

    fn execute_block(
        &mut self,
        block: &[Box<Stmt>],
        env: &Rc<RefCell<Environment>>,
    ) -> Result<(), ControlFlow>;

    fn output(&mut self) -> &mut dyn Write;
}

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call(
        &self,
        context: &mut dyn ExecutionContext,
        arguments: &[Cell],
    ) -> Result<Cell, RuntimeError>;
}

pub fn as_callable<T>(value: Rc<T>) -> Rc<dyn Callable>
where
    T: Callable + 'static,
{
    value
}

#[allow(clippy::vtable_address_comparisons)]
pub fn ptr_eq(left: &dyn Callable, right: &dyn Callable) -> bool {
    ptr::eq(left, right)
}
