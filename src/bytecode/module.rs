use crate::bytecode::chunk::Chunk;
use crate::bytecode::constant_pool::ConstantPool;
use crate::bytecode::Constant;

type ChunkIndex = usize;
type ConstantIndex = usize;

#[derive(Debug, Clone)]
pub struct Module {
    pub chunks: Vec<Chunk>,
    pub constants: ConstantPool,
}

impl Module {
    pub fn new() -> Self {
        Module {
            chunks: vec![],
            constants: ConstantPool::new(),
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

    pub fn insert_constant(&mut self, constant: Constant) -> ConstantIndex {
        self.constants.insert(constant)
    }

    pub fn get_constant(&self, index: usize) -> &Constant {
        self.constants.get(index)
    }
}
