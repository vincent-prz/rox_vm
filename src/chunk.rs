use crate::value::{print_value, Value};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum OpCode {
    OpConstant,
    OpReturn,
}

impl OpCode {
    pub fn new(byte: u8) -> Self {
        byte.try_into().expect("Could not decode byte")
    }
}

// allows cast from u8 to OpCode
impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == OpCode::OpConstant as u8 => Ok(OpCode::OpConstant),
            x if x == OpCode::OpReturn as u8 => Ok(OpCode::OpReturn),
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
        self.constants[address as usize]
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
    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        let current_lineno = self.line_info.get_lineno(offset).unwrap();
        if offset > 0 && current_lineno == self.line_info.get_lineno(offset - 1).unwrap() {
            print!("   | ");
        } else {
            print!("{} ", current_lineno);
        }

        let instruction: &OpCode = &OpCode::new(self.read_byte(offset));
        match instruction {
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),
            OpCode::OpConstant => self.constant_instruction("OP_CONSTANT", offset),
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_addr = self.code[offset + 1];
        print!("{:<16} {} '", name, constant_addr);
        print_value(self.constants[constant_addr as usize]);
        println!("'");
        offset + 2
    }
}
