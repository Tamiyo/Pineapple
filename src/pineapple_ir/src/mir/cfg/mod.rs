use crate::mir::cfg::basic_block::BasicBlock;
use crate::mir::Label;
use std::cell::RefCell;
use std::rc::Rc;

use crate::mir::Stmt;

pub mod basic_block;

type Statement = Rc<RefCell<Stmt>>;
type StatementIndex = usize;
type BlockIndex = usize;

pub struct CFG {
    entry_label: Label,
    blocks: Vec<BasicBlock>,
    statements: Vec<Statement>,
}
