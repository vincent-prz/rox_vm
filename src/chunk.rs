// NOTE: the size of an OpCode is 1 byte. It can be checked with mem::size_of
#[derive(Debug)]
pub enum OpCode {
    OpReturn,
}

pub struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { code: Vec::new() }
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }

    pub fn write(&mut self, op_code: OpCode) {
        self.code.push(op_code);
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset: usize = 0;
        while offset < self.count() {
            offset = self.disassemble_instruction(offset);
        }
    }
    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        let instruction = &self.code[offset];
        match instruction {
            OpCode::OpReturn => Chunk::simple_instruction("OP_RETURN", offset),
        }
    }

    fn simple_instruction(name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }
}
