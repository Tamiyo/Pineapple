use pineapple_ir::mir::Expr;
use pineapple_ir::mir::Label;
use pineapple_ir::mir::Oper;

type StatementIndex = usize;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BlockEntry {
    Entry(usize),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockExit {
    Exit(usize),
    None,
}

#[derive(Debug)]
pub struct BasicBlock {
    pub index: usize,
    pub entry: BlockEntry,
    pub statements: Vec<StatementIndex>,
    pub exit: BlockExit,
}

impl BasicBlock {
    pub fn new(index: usize) -> Self {
        BasicBlock {
            index,
            entry: BlockEntry::None,
            statements: vec![],
            exit: BlockExit::None,
        }
    }
}
