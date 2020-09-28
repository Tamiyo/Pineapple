use crate::bytecode::Instruction;
use pineapple_ir::mir::Label;
use pineapple_ir::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub label: Label,
    pub instructions: Vec<Instruction>,
}

impl Chunk {
    pub fn new(label: Label) -> Self {
        Chunk {
            label,
            instructions: vec![],
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LabelLocation {
    pub chunk_index: usize,
    pub instruction_index: usize,
}

impl LabelLocation {
    pub fn new(chunk_index: usize, instruction_index: usize) -> Self {
        LabelLocation {
            chunk_index,
            instruction_index,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Module {
    pub chunks: Vec<Chunk>,
    pub values: Vec<Value>,
    pub labels: HashMap<Label, LabelLocation>,
}

impl Module {
    pub fn add_chunk(&mut self, label: Label) {
        self.chunks.push(Chunk::new(label));
    }

    pub fn add_label(&mut self, label: &Label) {
        let c = self.chunks.len() - 1;
        let s = self.chunks.last().unwrap().instructions.len();

        self.labels.insert(*label, LabelLocation::new(c, s));
        self.add_instruction(Instruction::LABEL(*label));
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.chunks.last_mut().unwrap().add_instruction(instruction);
    }

    pub fn add_value(&mut self, value: Value) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }
}
