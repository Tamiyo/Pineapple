use crate::bytecode::instruction::Instruction;

/**
 *  [Chunk]
 *
 *  Chunk represents a "chunk" of bytecode instructions, that is,
 *  a list of compiled bytecode instructions that the virtual
 *  machine will execute.
 *
 *  Chunks can be thought of as independent scopes in the sense that
 *  each chunk contains the bytecode that needs to get executed by a
 *  given executable call, i.e. functions and classes get their own
 *  chunks since they both have instructions that need to
 *  get executed at different parts of the program.
 */
#[derive(Debug)]
pub struct Chunk {
    /**
     *  Vector of the instrructions that the chunk contains.
     */
    pub instructions: Vec<Instruction>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            instructions: vec![],
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> usize {
        self.instructions.push(instruction);
        self.instructions.len() - 1
    }
}
