use rox::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant, 123);
    chunk.write(OpCode::OpReturn as u8, 123);
    chunk.disassemble("test chunk")
}
