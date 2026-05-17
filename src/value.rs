use std::{
    cell::RefCell,
    fmt,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::chunk::Chunk;

#[derive(Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Str(String),
    Function(Function),
    Closure(Closure),
    NativeFunction(NativeFunction),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Function(function) => write!(f, "<fn {}>", function.name),
            Value::Closure(closure) => write!(f, "<fn {}>", closure.function.name),
            Value::NativeFunction(function) => write!(f, "<fn {}>", function.name),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Number(n) => *n == 0.0,
            Value::Boolean(b) => !b,
            Value::Str(s) => s == "",
            Value::Function(_) => false,
            Value::Closure(_) => false,
            Value::NativeFunction(_) => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsey()
    }
}

#[derive(Clone, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Rc<RefCell<Chunk>>,
    pub name: String,
    pub up_values: Vec<UpValue>,
}

impl Function {
    pub fn new(name: String, arity: usize) -> Self {
        Function {
            arity,
            name,
            chunk: Rc::new(RefCell::new(Chunk::new())),
            up_values: vec![],
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct UpValue {
    pub index: u8,
    pub is_local: bool,
}

impl UpValue {
    pub fn new(index: u8, is_local: bool) -> Self {
        UpValue { index, is_local }
    }
}

#[derive(Clone, PartialEq)]
pub struct Closure {
    pub function: Function,
    pub up_values: Vec<Rc<RefCell<RuntimeUpValue>>>,
}

impl Closure {
    pub fn new(function: Function) -> Self {
        Closure {
            function,
            up_values: vec![],
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum RuntimeUpValue {
    OpenUpValue(usize), // this will be the index on the stack
    ClosedUpValue(Value),
}

#[derive(Clone, PartialEq)]
pub struct NativeFunction {
    pub arity: usize,
    pub name: String,
    implementation: NativeFunctionImpl,
}

impl NativeFunction {
    pub fn call(&self, arg_count: usize, args: &[Value]) -> Result<Value, String> {
        self.implementation.call(arg_count, args)
    }
}

#[derive(Clone, PartialEq)]
enum NativeFunctionImpl {
    NativeClock,
}

impl NativeFunctionImpl {
    fn call(&self, arg_count: usize, args: &[Value]) -> Result<Value, String> {
        match self {
            NativeFunctionImpl::NativeClock => clock_native(arg_count, args),
        }
    }
}

pub fn get_clock_native_func() -> NativeFunction {
    NativeFunction {
        arity: 0,
        name: String::from("clock"),
        implementation: NativeFunctionImpl::NativeClock,
    }
}

fn clock_native(_arg_count: usize, _args: &[Value]) -> Result<Value, String> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let value = Value::Number(since_the_epoch.as_secs() as f64);
    Ok(value)
}
