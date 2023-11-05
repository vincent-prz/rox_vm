use rox::chunk::{Chunk, OpCode};
use rox::vm::VM;

fn main() {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(constant, 123);
    chunk.write(OpCode::OpReturn as u8, 123);
    vm.interpret(&chunk);
}
