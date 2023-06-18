use std::ops::Index;

pub type Value = f64;

#[derive(Debug)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

impl<T> Index<T> for ValueArray
where
    usize: From<T>,
{
    type Output = Value;

    fn index(&self, index: T) -> &Self::Output {
        &self.values[usize::from(index)]
    }
}
