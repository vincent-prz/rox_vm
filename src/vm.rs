use std::cell::{Ref, RefCell};
use std::collections::HashMap;

use crate::chunk::{Chunk, OpCode};
use crate::value::{Function, Value};

static FRAMES_MAX: usize = 64;
// static UINT8_COUNT: usize = 256;
// static STACK_MAX: usize =  FRAMES_MAX * UINT8_COUNT;

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
    pub fn new(function: Function) -> Self {
        let mut frames = Vec::with_capacity(FRAMES_MAX);
        let current_frame = CallFrame::new(&function,0, 0);
        frames.push(current_frame);
        VM {
            stack: vec![Value::Function(function)],
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self) -> Result<(), RuntimeError> {
        self.run()
    }

    fn run_callframe(&mut self, frame: RefCell<CallFrame>) -> Result<(), RuntimeError> {
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
                self.get_chunk()
                    .disassemble_instruction(frame.ip);
            }
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
                    let result = self.pop();
                    // remove param arguments from the stack.
                    self.stack.truncate(frame.borrow().slots_start_index);
                    self.stack.push(result);
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
                    self.stack.push(local_value)
                }
                OpCode::OpSetLocal => {
                    let local_index = self.read_byte();
                    let usize_index: usize = local_index.into();
                    self.stack[usize_index] = self.peek(0).clone();
                }
                OpCode::OpJump => {
                    frame.borrow_mut().ip += self.read_short() as usize;
                }
                OpCode::OpJumpIfTrue => {
                    let condition_is_truthy = self.peek(0).is_truthy();
                    let jump: usize = self.read_short() as usize;
                    if condition_is_truthy {
                        frame.borrow_mut().ip += jump;
                    }
                }
                OpCode::OpJumpIfFalse => {
                    let condition_is_falsey = self.peek(0).is_falsey();
                    let jump: usize = self.read_short() as usize;
                    if condition_is_falsey {
                        frame.borrow_mut().ip += jump;
                    }
                }
                OpCode::OpLoop => {
                    frame.borrow_mut().ip -= self.read_short() as usize;
                }
                OpCode::OpCall => {
                    let nb_args = self.read_byte();
                    let callee = self.peek(nb_args as usize);
                    match callee {
                        Value::Function(function) => {
                            let arity = function.arity;
                            let new_frame = CallFrame {
                                function,
                                ip: 0,
                                // Subtle: the `- arity` part is for the overlapping of callframes
                                // windows on the stack, see 24.5.1. - 1 is for the slot reserved for the fucntion itself
                                slots_start_index: self.stack.len() - arity - 1,
                            };
                            self.run_callframe(RefCell::new(new_frame));
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

    fn get_chunk(&self, frame: &CallFrame) -> Ref<Chunk> {
        let frame = frame;
        let function = &self.stack[frame.slots_start_index];
        match function {
            Value::Function(func) => func.chunk.borrow(),
            _ => panic!("Tried to get chunk of non function"),
        }
    }

    fn read_byte(&mut self) -> u8 {
        let result = self
            .get_chunk()
            .read_byte(frame.ip);
        frame.ip += 1;
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
        let slots_start_index = frame.slots_start_index;
        self.stack[usize_index + slots_start_index].clone()
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
        let lineno = self
            .get_chunk()
            .get_lineno(frame.ip - 1);
        self.reset_stack();
        RuntimeError {
            msg: format!("{}\n[line {}] in script", msg, lineno),
        }
    }
}

pub struct RuntimeError {
    pub msg: String,
}
