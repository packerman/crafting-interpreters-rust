use crate::vm::chunk::OpCode;

use super::chunk::Chunk;

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.count() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        let instruction = self.code()[offset];
        match instruction {
            _ if instruction == OpCode::Return as u8 => Self::simple_instruction("RETURN", offset),
            _ if instruction == OpCode::Constant as u8 => {
                self.constant_instruction("CONSTANT", offset)
            }
            _ => {
                println!("Unknown opcode {}", instruction);
                offset + 1
            }
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code()[offset + 1];
        println!("{:>16} {:4} {}", name, constant, self.constants()[constant]);
        offset + 2
    }

    fn simple_instruction(name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
}
