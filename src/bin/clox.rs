use crafting_interpreters_rust::vm::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant);
    chunk.write(constant as u8);

    chunk.write(OpCode::Return);
    chunk.disassemble("test chunk");
}
