use std::{cell::RefCell, rc::Rc};

use pineapple_ir::mir::Stmt;

type Statement = Rc<RefCell<Stmt>>;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockEntry {
    Entry(Statement),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockExit {
    Exit(Statement),
    None,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub index: usize,
    pub entry: BlockEntry,
    pub statements: Vec<Rc<RefCell<Stmt>>>,
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
