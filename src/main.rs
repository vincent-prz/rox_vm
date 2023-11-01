use rox::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::OpReturn);
    chunk.disassemble("test chunk")
}
