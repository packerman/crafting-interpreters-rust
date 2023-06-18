use super::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return,
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write<T>(&mut self, value: T)
    where
        u8: From<T>,
    {
        self.code.push(u8::from(value));
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.count() - 1
    }

    pub fn constants(&self) -> &ValueArray {
        &self.constants
    }
}
