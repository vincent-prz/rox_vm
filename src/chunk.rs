use crate::value::{print_value, Value};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum OpCode {
    OpConstant,
    OpReturn,
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
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }

    pub fn write(&mut self, op_code: u8) {
        self.code.push(op_code);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1)
            .try_into()
            .expect("Constant index didn't fit in byte")
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

        let instruction: &OpCode = &self.code[offset].try_into().expect("Could not decode byte");
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
        print!("{} {:04} '", name, constant_addr);
        print_value(self.constants[constant_addr as usize]);
        println!("'");
        offset + 2
    }
}
