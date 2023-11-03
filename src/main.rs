use rox::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8);
    chunk.write(constant);
    chunk.write(OpCode::OpReturn as u8);
    chunk.disassemble("test chunk")
}
