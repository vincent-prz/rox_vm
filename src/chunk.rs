use crate::value::Value;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum OpCode {
    OpConstant,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNegate,
    OpPrint,
    OpReturn,
    OpTrue,
    OpFalse,
    OpNot,
    OpAnd,
    OpOr,
    OpEqualEqual,
    OpBangEqual,
    OpLess,
    OpLessEqual,
    OpGreater,
    OpGreaterEqual,
    OpDefineGlobal,
    OpGetGlobal,
    OpPop,
    OpPopN,
    OpGetLocal,
    OpEof,
}

impl OpCode {
    pub fn new(byte: u8) -> Self {
        // [perf] - try_into might incurr an avoidable perf penalty
        byte.try_into()
            .expect(&format!("Could not decode byte {}", byte))
    }
}

// allows cast from u8 to OpCode
impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == OpCode::OpConstant as u8 => Ok(OpCode::OpConstant),
            x if x == OpCode::OpAdd as u8 => Ok(OpCode::OpAdd),
            x if x == OpCode::OpSubtract as u8 => Ok(OpCode::OpSubtract),
            x if x == OpCode::OpMultiply as u8 => Ok(OpCode::OpMultiply),
            x if x == OpCode::OpDivide as u8 => Ok(OpCode::OpDivide),
            x if x == OpCode::OpNegate as u8 => Ok(OpCode::OpNegate),
            x if x == OpCode::OpPrint as u8 => Ok(OpCode::OpPrint),
            x if x == OpCode::OpReturn as u8 => Ok(OpCode::OpReturn),
            x if x == OpCode::OpTrue as u8 => Ok(OpCode::OpTrue),
            x if x == OpCode::OpFalse as u8 => Ok(OpCode::OpFalse),
            x if x == OpCode::OpNot as u8 => Ok(OpCode::OpNot),
            x if x == OpCode::OpAnd as u8 => Ok(OpCode::OpAnd),
            x if x == OpCode::OpOr as u8 => Ok(OpCode::OpOr),
            x if x == OpCode::OpEqualEqual as u8 => Ok(OpCode::OpEqualEqual),
            x if x == OpCode::OpBangEqual as u8 => Ok(OpCode::OpBangEqual),
            x if x == OpCode::OpLess as u8 => Ok(OpCode::OpLess),
            x if x == OpCode::OpLessEqual as u8 => Ok(OpCode::OpLessEqual),
            x if x == OpCode::OpGreater as u8 => Ok(OpCode::OpGreater),
            x if x == OpCode::OpGreaterEqual as u8 => Ok(OpCode::OpGreaterEqual),
            x if x == OpCode::OpDefineGlobal as u8 => Ok(OpCode::OpDefineGlobal),
            x if x == OpCode::OpGetGlobal as u8 => Ok(OpCode::OpGetGlobal),
            x if x == OpCode::OpPop as u8 => Ok(OpCode::OpPop),
            x if x == OpCode::OpPopN as u8 => Ok(OpCode::OpPopN),
            x if x == OpCode::OpGetLocal as u8 => Ok(OpCode::OpGetLocal),
            x if x == OpCode::OpEof as u8 => Ok(OpCode::OpEof),
            _ => Err(()),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    line_info: LineInfo,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            line_info: LineInfo::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }

    pub fn write(&mut self, op_code: u8, lineno: usize) {
        self.code.push(op_code);
        self.line_info.add(self.count() - 1, lineno);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1)
            .try_into()
            .expect("Constant index didn't fit in byte")
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn read_constant(&self, address: u8) -> Value {
        // [perf] what's the perf impact of this clone ?
        self.constants[address as usize].clone()
    }

    pub fn get_lineno(&self, offset: usize) -> usize {
        self.line_info
            .get_lineno(offset)
            .expect(&format!("Couldn't retrieve lineno for offset {}", offset))
    }
}

/// Line info is encoded with tuples like representing `(offset, lineno).`
/// where offset is the first offset comprised in lineno.
/// Assumption: offsets are added in ascending order.
struct LineInfo {
    info: Vec<(usize, usize)>,
}

impl LineInfo {
    fn new() -> LineInfo {
        LineInfo { info: Vec::new() }
    }

    fn add(&mut self, offset: usize, lineno: usize) {
        match self.info.last() {
            None => {
                self.info.push((offset, lineno));
            }
            Some((_, current_lineno)) => {
                if lineno > *current_lineno {
                    self.info.push((offset, lineno))
                }
            }
        }
    }

    fn get_lineno(&self, offset: usize) -> Option<usize> {
        for index in 0..self.info.len() {
            let (current_offset, current_lineno) = self.info[index];
            if offset == current_offset {
                return Some(current_lineno);
            }
            if offset < current_offset {
                if index > 0 {
                    return Some(self.info[index - 1].1);
                } else {
                    return None;
                }
            }
        }
        match self.info.last() {
            None => None,
            Some((_, last_lineno)) => Some(*last_lineno),
        }
    }
}

/// debug implementation
impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset: usize = 0;
        while offset < self.count() {
            offset = self.disassemble_instruction(offset);
        }
    }
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        let current_lineno = self.line_info.get_lineno(offset).unwrap();
        if offset > 0 && current_lineno == self.line_info.get_lineno(offset - 1).unwrap() {
            print!("   | ");
        } else {
            print!("{:4} ", current_lineno);
        }

        let instruction: &OpCode = &OpCode::new(self.read_byte(offset));
        match instruction {
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),
            OpCode::OpAdd => self.simple_instruction("OP_ADD", offset),
            OpCode::OpSubtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::OpMultiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::OpDivide => self.simple_instruction("OP_DIVIDE", offset),
            OpCode::OpNegate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::OpPrint => self.simple_instruction("OP_PRINT", offset),
            OpCode::OpConstant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::OpTrue => self.simple_instruction("OP_TRUE", offset),
            OpCode::OpFalse => self.simple_instruction("OP_FALSE", offset),
            OpCode::OpNot => self.simple_instruction("OP_NOT", offset),
            OpCode::OpAnd => self.simple_instruction("OP_AND", offset),
            OpCode::OpOr => self.simple_instruction("OP_OR", offset),
            OpCode::OpEqualEqual => self.simple_instruction("OP_EQUAL_EQUAL", offset),
            OpCode::OpBangEqual => self.simple_instruction("OP_BANG_EQUAL", offset),
            OpCode::OpLess => self.simple_instruction("OP_LESS", offset),
            OpCode::OpLessEqual => self.simple_instruction("OP_LESS_EQUAL", offset),
            OpCode::OpGreater => self.simple_instruction("OP_GREATER", offset),
            OpCode::OpGreaterEqual => self.simple_instruction("OP_GREATER_EQUAL", offset),
            OpCode::OpDefineGlobal => self.constant_instruction("OP_DEFINE_GLOBAL", offset),
            OpCode::OpGetGlobal => self.constant_instruction("OP_GET_GLOBAL", offset),
            OpCode::OpPop => self.simple_instruction("OP_POP", offset),
            OpCode::OpPopN => self.instruction_with_operand("OP_POPN", offset),
            OpCode::OpGetLocal => self.instruction_with_operand("OP_GET_LOCAL", offset),
            OpCode::OpEof => self.simple_instruction("OP_EOF", offset),
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
    fn instruction_with_operand(&self, name: &str, offset: usize) -> usize {
        let operand = self.code[offset + 1];
        println!("{:<16} {}", name, operand);
        offset + 2
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_addr = self.code[offset + 1];
        print!("{:<16} {} '", name, constant_addr);
        print!("{}", self.constants[constant_addr as usize]);
        println!("'");
        offset + 2
    }
}
