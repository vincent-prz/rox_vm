use std::cell::Ref;
use std::collections::HashMap;

use crate::chunk::{Chunk, OpCode};
use crate::value::{Function, Value};

pub struct VM {
    // [perf] likewise, using stack.len() instead of a pointer to keep track of the top.
    // [perf] should we used a fixed size array ?
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

// NOTE - to retrieve the callframe function, we can use `stack[slots_start_index]`
// this avoids the need to have a `function` field and tricky lifetime issues
struct CallFrame<'a> {
    function: &'a Function,
    // NOTE - [perf] not really an instruction pointer as in the book, but a mere counter
    // This is in order to avoid using unsafe Rust. TODO: benchmark
    ip: usize,
    slots_start_index: usize,
}

impl<'a> CallFrame<'a> {
    const fn new(function: &'a Function, ip: usize, slots_start_index: usize) -> Self {
        CallFrame {
            function,
            ip,
            slots_start_index,
        }
    }
}

macro_rules! binary_op {
    ($self:expr, $op:tt, $valueType:expr, $frame:expr) => {{
        let b = $self.pop();
        let a = $self.pop();
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => {
                $self.push($valueType(x $op y));
            },
            _ => {
                Err($self.runtime_error("Operands must be numbers".to_string(), $frame))?;
            }
        }
    }};
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, script_function: Function) -> Result<(), RuntimeError> {
        self.stack.push(Value::Function(script_function.clone()));
        let mut first_frame = CallFrame::new(&script_function, 0, 0);
        self.run_callframe(&mut first_frame)
    }

    fn run_callframe(&mut self, frame: &mut CallFrame) -> Result<(), RuntimeError> {
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
                // let func_name = match &frame.function.name {
                //     Some(func_name) => func_name.clone(),
                //     None => String::from("<script>"),
                // };
                // print!("{}::", func_name);
                self.get_chunk().disassemble_instruction(frame.ip);
            }
            let instruction = self.read_byte(frame).try_into().unwrap();
            match instruction {
                OpCode::OpConstant => {
                    let constant = self.read_constant(frame);
                    self.push(constant);
                }
                OpCode::OpNegate => {
                    let value = self.pop();
                    match value {
                        Value::Number(number) => self.push(Value::Number(-number)),
                        _ => {
                            Err(self.runtime_error("Operand must be a number".to_string(), frame))?
                        }
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
                                frame,
                            ))?;
                        }
                    }
                }
                OpCode::OpSubtract => binary_op!(self, -, Value::Number, frame),
                OpCode::OpMultiply => binary_op!(self, *, Value::Number, frame),
                OpCode::OpDivide => binary_op!(self, /, Value::Number, frame),
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
                OpCode::OpLess => binary_op!(self, <, Value::Boolean, frame),
                OpCode::OpLessEqual => binary_op!(self, <=, Value::Boolean, frame),
                OpCode::OpGreater => binary_op!(self, >, Value::Boolean, frame),
                OpCode::OpGreaterEqual => binary_op!(self, >=, Value::Boolean, frame),
                OpCode::OpReturn => {
                    let result = self.pop();
                    // remove param arguments from the stack.
                    self.stack.truncate(frame.slots_start_index);
                    self.stack.push(result);
                    return Ok(());
                }
                OpCode::OpTrue => self.push(Value::Boolean(true)),
                OpCode::OpFalse => self.push(Value::Boolean(false)),
                OpCode::OpNot => {
                    let value = self.pop();
                    match value {
                        Value::Boolean(b) => self.push(Value::Boolean(!b)),
                        _ => {
                            Err(self.runtime_error("Operand must be a boolean".to_string(), frame))?
                        }
                    }
                }
                OpCode::OpPrint => {
                    println!("{}", self.pop());
                }
                OpCode::OpDefineGlobal => {
                    let value = self.pop();
                    let constant = self.read_constant(frame);
                    if let Value::Str(constant) = constant {
                        self.globals.insert(constant, value);
                    } else {
                        Err(self.runtime_error("Expected string constant".to_string(), frame))?;
                    }
                }
                OpCode::OpGetGlobal => {
                    let constant = self.read_constant(frame);
                    if let Value::Str(constant) = constant {
                        if let Some(value) = self.globals.get(&constant) {
                            self.push(value.clone());
                        } else {
                            Err(self.runtime_error(
                                format!("Undefined variable '{}'", constant),
                                frame,
                            ))?;
                        }
                    } else {
                        Err(self.runtime_error("Expected string constant".to_string(), frame))?;
                    }
                }
                OpCode::OpSetGlobal => {
                    let constant = self.read_constant(frame);
                    if let Value::Str(constant) = constant {
                        if self.globals.contains_key(&constant) {
                            self.globals.insert(constant, self.peek(0).clone());
                        } else {
                            Err(self.runtime_error(
                                format!("Cannot assign undefined variable {}.", constant),
                                frame,
                            ))?;
                        }
                    } else {
                        Err(self.runtime_error("Expected string constant".to_string(), frame))?;
                    }
                }
                OpCode::OpPop => {
                    self.pop();
                }
                OpCode::OpPopN => {
                    let nb_elems_to_pop = self.read_byte(frame);
                    self.pop_n(nb_elems_to_pop);
                }
                OpCode::OpGetLocal => {
                    let local_index = self.read_byte(frame);
                    let local_value = self.get_local(local_index, frame);
                    self.stack.push(local_value)
                }
                OpCode::OpSetLocal => {
                    let local_index = self.read_byte(frame);
                    let usize_index: usize = local_index.into();
                    self.stack[usize_index] = self.peek(0).clone();
                }
                OpCode::OpJump => {
                    frame.ip += self.read_short(frame) as usize;
                }
                OpCode::OpJumpIfTrue => {
                    let condition_is_truthy = self.peek(0).is_truthy();
                    let jump: usize = self.read_short(frame) as usize;
                    if condition_is_truthy {
                        frame.ip += jump;
                    }
                }
                OpCode::OpJumpIfFalse => {
                    let condition_is_falsey = self.peek(0).is_falsey();
                    let jump: usize = self.read_short(frame) as usize;
                    if condition_is_falsey {
                        frame.ip += jump;
                    }
                }
                OpCode::OpLoop => {
                    frame.ip -= self.read_short(frame) as usize;
                }
                OpCode::OpCall => {
                    let nb_args = self.read_byte(frame);
                    let callee = self.peek(nb_args as usize);
                    match callee {
                        Value::Function(function) => {
                            let arity = function.arity;
                            let mut new_frame = CallFrame {
                                function: &function.clone(),
                                ip: 0,
                                // Subtle: the `- arity` part is for the overlapping of callframes
                                // windows on the stack, see 24.5.1. - 1 is for the slot reserved for the function itself
                                slots_start_index: self.stack.len() - arity - 1,
                            };
                            self.run_callframe(&mut new_frame)?;
                        }
                        value => todo!(), // FIXME
                    }
                }
                OpCode::OpEof => {
                    return Ok(());
                }
            }
        }
    }

    fn get_chunk<'a>(&self, frame: &CallFrame<'a>) -> Ref<'a, Chunk> {
        frame.function.chunk.borrow()
    }

    fn read_byte(&mut self, frame: &mut CallFrame) -> u8 {
        let result = self.get_chunk(frame).read_byte(frame.ip);
        frame.ip += 1;
        result
    }

    fn read_constant(&mut self, frame: &mut CallFrame) -> Value {
        let byte = self.read_byte(frame);
        self.get_chunk(frame).read_constant(byte)
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

    fn get_local(&self, index: u8, frame: &CallFrame) -> Value {
        let usize_index: usize = index.into();
        let slots_start_index = frame.slots_start_index;
        self.stack[usize_index + slots_start_index].clone()
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn read_short(&mut self, frame: &mut CallFrame) -> u16 {
        let x: u16 = self.read_byte(frame).into();
        let y: u16 = self.read_byte(frame).into();
        (x << 8) | y
    }

    fn runtime_error(&mut self, msg: String, frame: &CallFrame) -> RuntimeError {
        let lineno = self.get_chunk(frame).get_lineno(frame.ip - 1);
        self.reset_stack();
        RuntimeError {
            msg: format!("{}\n[line {}] in script", msg, lineno),
        }
    }
}

pub struct RuntimeError {
    pub msg: String,
}
