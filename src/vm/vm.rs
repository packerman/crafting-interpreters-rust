use super::{
    chunk::{Chunk, OpCode},
    value::Value,
};

pub struct Vm {
    chunk: Chunk,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            match self.read_byte() {
                instruction if instruction == OpCode::Constant as u8 => {
                    let constant = self.read_constant();
                    println!("{}", constant);
                }
                instruction if instruction == OpCode::Return as u8 => return InterpretResult::Ok,
                _ => {}
            }
        }
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let result = self.chunk[self.ip];
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();
        self.chunk.constants()[index]
    }
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}
