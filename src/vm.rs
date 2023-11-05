use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value};

pub struct VM<'a> {
    chunk: Option<&'a Chunk>,
    // NOTE - [perf] not really an instruction pointer as in the book, but a mere counter
    // This is in order to avoid using unsafe Rust. TODO: benchmark
    ip: usize,
    // [perf] likewise, using stack.len() instead of a pointer to keep track of the top.
    stack: Vec<Value>,
}

macro_rules! binary_op {
    ($self:expr, $op:expr) => {{
        let b = $self.pop();
        let a = $self.pop();
        $self.push($op(a, b));
    }};
}
impl<'a> VM<'a> {
    pub fn new() -> Self {
        VM {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, chunk: &'a Chunk) -> InterpretResult {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(feature = "debugTraceExecution")]
            {
                print!("          ");
                for value in &self.stack {
                    print!("[ ");
                    print_value(*value);
                    print!(" ]");
                }
                println!("");
                self.unwrap_chunk().disassemble_instruction(self.ip);
            }
            let instruction = OpCode::new(self.read_byte());
            match instruction {
                OpCode::OpConstant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::OpNegate => {
                    let value = self.pop();
                    self.push(-value);
                }
                OpCode::OpAdd => binary_op!(self, |x: Value, y: Value| -> Value { x + y }),
                OpCode::OpSubtract => binary_op!(self, |x: Value, y: Value| -> Value { x - y }),
                OpCode::OpMultiply => binary_op!(self, |x: Value, y: Value| -> Value { x * y }),
                OpCode::OpDivide => binary_op!(self, |x: Value, y: Value| -> Value { x / y }),
                OpCode::OpReturn => {
                    print_value(self.pop());
                    println!("");
                    return InterpretResult::InterpretOk;
                }
            }
        }
    }

    /// helper to avoid dealing with Option. This should be safe to call within
    /// the context of an interpret run.
    fn unwrap_chunk(&self) -> &Chunk {
        self.chunk.expect("Expected chunk to be set")
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.unwrap_chunk().read_byte(self.ip);
        self.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.unwrap_chunk().read_constant(byte)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Tried to pop on empty stack")
    }
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}
