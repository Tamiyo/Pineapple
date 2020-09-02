use crate::bytecode::OpCode;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub opcodes: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk { opcodes: vec![] }
    }

    pub fn add_instruction(&mut self, opcode: OpCode) -> usize {
        self.opcodes.push(opcode);
        self.opcodes.len() - 1
    }
}
