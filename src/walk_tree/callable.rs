use super::{interpreter::Interpreter, value::Cell};

pub trait Callable<'a, W> {
    fn call(&self, interpreter: &Interpreter<'a, W>, arguments: &[Cell]) -> Cell;
}
