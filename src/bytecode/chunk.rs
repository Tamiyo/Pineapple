use crate::bytecode::Instruction;
use crate::bytecode::Label;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<Label, usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            instructions: vec![],
            labels: HashMap::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
       if let Instruction::LABEL(label) = instruction {
            self.labels.insert(label, self.instructions.len());
       }
       self.instructions.push(instruction);
    }
}
