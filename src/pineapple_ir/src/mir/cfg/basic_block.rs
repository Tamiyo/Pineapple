use crate::mir::Expr;
use crate::mir::Label;
use crate::mir::Oper;

type StatementIndex = usize;

pub enum BlockEntry {
    Label(Label),
}

pub enum BlockExit {
    Jump(Label),
    CJump(Expr, Label),
    Return(Option<Oper>),
}

pub struct BasicBlock {
    entry: BlockEntry,
    statements: Vec<StatementIndex>,
    exit: BlockExit,
}
