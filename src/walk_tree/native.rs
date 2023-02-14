use std::{sync::Arc, time::SystemTime};

use super::{
    callable::{Callable, ExecutionContext},
    error::RuntimeError,
    value::Cell,
};

#[derive(Debug)]
struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _context: &mut dyn ExecutionContext,
        _argumentss: &[Cell],
    ) -> Result<Cell, RuntimeError> {
        let duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| RuntimeError::from(String::from("SystemTime before UNIX EPOCH!")))?;
        Ok(Cell::from(duration.as_secs_f64()))
    }
}

pub fn clock() -> Cell {
    let value: Arc<dyn Callable> = Arc::new(Clock);
    Cell::from(value)
}

#[derive(Debug)]
struct Print;

impl Callable for Print {
    fn arity(&self) -> usize {
        1
    }

    fn call(
        &self,
        context: &mut dyn ExecutionContext,
        arguments: &[Cell],
    ) -> Result<Cell, RuntimeError> {
        writeln!(context.output(), "{}", arguments[0])
            .map_err(|err| RuntimeError::from(format!("Print error: {err}")))?;
        Ok(Cell::from(()))
    }
}

pub fn print() -> Cell {
    let value: Arc<dyn Callable> = Arc::new(Print);
    Cell::from(value)
}
