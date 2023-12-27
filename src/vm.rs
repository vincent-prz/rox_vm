use std::collections::HashMap;

use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

pub struct VM {
    chunk: Option<Chunk>,
    // NOTE - [perf] not really an instruction pointer as in the book, but a mere counter
    // This is in order to avoid using unsafe Rust. TODO: benchmark
    ip: usize,
    // [perf] likewise, using stack.len() instead of a pointer to keep track of the top.
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

macro_rules! binary_op {
    ($self:expr, $op:tt, $valueType:expr) => {{
        let b = $self.pop();
        let a = $self.pop();
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => {
                $self.push($valueType(x $op y));
            },
            _ => {
                Err($self.runtime_error("Operands must be numbers".to_string()))?;
            }
        }
    }};
}

macro_rules! logical_op {
    ($self:expr, $op:tt) => {{
        let b = $self.pop();
        let a = $self.pop();
        match (a, b) {
            (Value::Boolean(x), Value::Boolean(y)) => {
                $self.push(Value::Boolean(x $op y));
            },
            _ => {
                Err($self.runtime_error("Operands must be booleans".to_string()))?;
            }
        }
    }};
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), RuntimeError> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            #[cfg(feature = "debugTraceExecution")]
            {
                print!("          ");
                for value in &self.stack {
                    print!("[ ");
                    print!("{}", *value);
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
                    match value {
                        Value::Number(number) => self.push(Value::Number(-number)),
                        _ => Err(self.runtime_error("Operand must be a number".to_string()))?,
                    }
                }
                OpCode::OpAdd => {
                    let b = self.pop();
                    let a = self.pop();
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => {
                            self.push(Value::Number(x + y));
                        }
                        (Value::Str(x), Value::Str(y)) => {
                            self.push(Value::Str(format!("{}{}", x, y)));
                        }
                        _ => {
                            Err(self.runtime_error(
                                "Operands must be two numbers or two strings".to_string(),
                            ))?;
                        }
                    }
                }
                OpCode::OpSubtract => binary_op!(self, -, Value::Number),
                OpCode::OpMultiply => binary_op!(self, *, Value::Number),
                OpCode::OpDivide => binary_op!(self, /, Value::Number),
                OpCode::OpEqualEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Boolean(a == b));
                }
                OpCode::OpBangEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Boolean(a != b));
                }
                OpCode::OpLess => binary_op!(self, <, Value::Boolean),
                OpCode::OpLessEqual => binary_op!(self, <=, Value::Boolean),
                OpCode::OpGreater => binary_op!(self, >, Value::Boolean),
                OpCode::OpGreaterEqual => binary_op!(self, >=, Value::Boolean),
                OpCode::OpReturn => {
                    return Ok(());
                }
                OpCode::OpTrue => self.push(Value::Boolean(true)),
                OpCode::OpFalse => self.push(Value::Boolean(false)),
                OpCode::OpNot => {
                    let value = self.pop();
                    match value {
                        Value::Boolean(b) => self.push(Value::Boolean(!b)),
                        _ => Err(self.runtime_error("Operand must be a boolean".to_string()))?,
                    }
                }
                OpCode::OpAnd => logical_op!(self, &&),
                OpCode::OpOr => logical_op!(self, ||),
                OpCode::OpPrint => {
                    println!("{}", self.pop());
                }
                OpCode::OpDefineGlobal => {
                    let value = self.pop();
                    let constant = self.read_constant();
                    if let Value::Str(constant) = constant {
                        self.globals.insert(constant, value);
                    } else {
                        Err(self.runtime_error("Expected string constant".to_string()))?;
                    }
                }
                OpCode::OpGetGlobal => {
                    let constant = self.read_constant();
                    if let Value::Str(constant) = constant {
                        if let Some(value) = self.globals.get(&constant) {
                            self.push(value.clone());
                        } else {
                            Err(self.runtime_error(format!("Undefined variable '{}'", constant)))?;
                        }
                    } else {
                        Err(self.runtime_error("Expected string constant".to_string()))?;
                    }
                }
                OpCode::OpEof => {
                    return Ok(());
                }
            }
        }
    }

    /// helper to avoid dealing with Option. This should be safe to call within
    /// the context of an interpret run.
    fn unwrap_chunk(&self) -> &Chunk {
        self.chunk.as_ref().expect("Expected chunk to be set")
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

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn runtime_error(&mut self, msg: String) -> RuntimeError {
        let lineno = self.unwrap_chunk().get_lineno(self.ip - 1);
        self.reset_stack();
        RuntimeError {
            msg: format!("{}\n[line {}] in script", msg, lineno),
        }
    }
}

pub struct RuntimeError {
    pub msg: String,
}
