use crate::bytecode::chunk::Chunk;
use crate::bytecode::Label;
use crate::{bytecode::constant_pool::ConstantPool, core::value::Value};
use std::collections::HashMap;

type ChunkIndex = usize;
type ConstantIndex = usize;

#[derive(Debug, Clone)]
pub struct Module {
    pub chunks: Vec<Chunk>,
    pub constants: ConstantPool,
    pub named_labels: HashMap<usize, usize>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            chunks: vec![],
            constants: ConstantPool::new(),
            named_labels: HashMap::new(),
        }
    }

    pub fn add_chunk(&mut self) -> ChunkIndex {
        self.chunks.push(Chunk::new());
        self.chunks.len() - 1
    }

    pub fn get_chunk(&self, chunk_index: usize) -> &Chunk {
        &self.chunks[chunk_index]
    }

    pub fn get_chunk_mut(&mut self, chunk_index: usize) -> &mut Chunk {
        &mut self.chunks[chunk_index]
    }

    pub fn add_constant(&mut self, constant: Value) -> ConstantIndex {
        self.constants.insert(constant)
    }

    pub fn get_constant(&self, index: usize) -> &Value {
        self.constants.get(index)
    }
}
