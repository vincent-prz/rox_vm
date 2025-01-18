use std::collections::HashMap;

use crate::chunk::{Chunk, OpCode};
use crate::value::{Function, Value};

static FRAMES_MAX: usize = 64;

pub struct VM {
    frames: [CallFrame; FRAMES_MAX],
    // FIXME: remove this
    frame: CallFrame,
    frame_count: usize,
    // [perf] likewise, using stack.len() instead of a pointer to keep track of the top.
    // [perf] FIXME: use fixed length array instead
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

#[derive(Clone)]
struct CallFrame {
    function: Function,
    // NOTE - [perf] not really an instruction pointer as in the book, but a mere counter
    // This is in order to avoid using unsafe Rust. TODO: benchmark
    ip: usize,
    // [perf] FIXME: use fixed sized array
    slots: Vec<Value>,
}

impl CallFrame {
    const fn new() -> Self {
        CallFrame {function: Function::new(), ip:0, slots: Vec::new()}
    }
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

impl VM {
    pub fn new() -> Self {
        VM {
            frames: [const { CallFrame::new() }; FRAMES_MAX],
            frame: CallFrame::new(),
            frame_count: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, function: Function) -> Result<(), RuntimeError> {
        let mut frame = self.frames[self.frame_count];
        frame.function = function;
        frame.ip = function.chunk.code;
        frame.slots = self.stack;
        self.frame_count += 1;
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

            // [perf] FIXME: remove this clone
            self.frame = self.frames[self.frame_count - 1].clone();
            let instruction = self.read_byte().try_into().unwrap();
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
                OpCode::OpSetGlobal => {
                    let constant = self.read_constant();
                    if let Value::Str(constant) = constant {
                        if self.globals.contains_key(&constant) {
                            self.globals.insert(constant, self.peek(0).clone());
                        } else {
                            Err(self.runtime_error(format!(
                                "Cannot assign undefined variable {}.",
                                constant
                            )))?;
                        }
                    } else {
                        Err(self.runtime_error("Expected string constant".to_string()))?;
                    }
                }
                OpCode::OpPop => {
                    self.pop();
                }
                OpCode::OpPopN => {
                    let nb_elems_to_pop = self.read_byte();
                    self.pop_n(nb_elems_to_pop);
                }
                OpCode::OpGetLocal => {
                    let local_index = self.read_byte();
                    let local_value = self.get_local(local_index);
                    self.frame.slots.push(local_value);
                }
                OpCode::OpSetLocal => {
                    let local_index = self.read_byte();
                    let usize_index: usize = local_index.into();
                    self.stack[usize_index] = self.peek(0).clone();
                }
                OpCode::OpJump => {
                    self.frame.ip += self.read_short() as usize;
                }
                OpCode::OpJumpIfTrue => {
                    let condition_is_truthy = self.peek(0).is_truthy();
                    let jump: usize = self.read_short() as usize;
                    if condition_is_truthy {
                        self.frame.ip += jump;
                    }
                }
                OpCode::OpJumpIfFalse => {
                    let condition_is_falsey = self.peek(0).is_falsey();
                    let jump: usize = self.read_short() as usize;
                    if condition_is_falsey {
                        self.frame.ip += jump;
                    }
                }
                OpCode::OpLoop => {
                    self.frame.ip -= self.read_short() as usize;
                }
                OpCode::OpEof => {
                    return Ok(());
                }
            }
        }
    }

    fn get_chunk(&self) -> &Chunk {
        &self.frame.function.chunk
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.get_chunk().read_byte(self.frame.ip);
        self.frame.ip += 1;
        result
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.get_chunk().read_constant(byte)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn peek(&self, offset: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - offset]
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Tried to pop on empty stack")
    }

    fn pop_n(&mut self, nb_elem_to_pop: u8) {
        let new_len = self.stack.len() - <u8 as Into<usize>>::into(nb_elem_to_pop);
        self.stack.truncate(new_len);
    }

    fn get_local(&self, index: u8) -> Value {
        let usize_index: usize = index.into();
        self.stack[usize_index].clone()
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn read_short(&mut self) -> u16 {
        let x: u16 = self.read_byte().into();
        let y: u16 = self.read_byte().into();
        (x << 8) | y
    }

    fn runtime_error(&mut self, msg: String) -> RuntimeError {
        let lineno = self.get_chunk().get_lineno(self.frame.ip - 1);
        self.reset_stack();
        RuntimeError {
            msg: format!("{}\n[line {}] in script", msg, lineno),
        }
    }
}

pub struct RuntimeError {
    pub msg: String,
}
