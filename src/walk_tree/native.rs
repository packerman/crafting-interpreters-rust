use std::{sync::Arc, time::SystemTime};

use super::{
    callable::{Callable, Context},
    error::RuntimeError,
    value::Cell,
};

#[derive(Debug)]
struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _context: &mut dyn Context, _argumentss: &[Cell]) -> Result<Cell, RuntimeError> {
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
