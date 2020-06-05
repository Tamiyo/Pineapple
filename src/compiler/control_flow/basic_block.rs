use crate::compiler::three_address::component::*;

#[derive(Clone)]
pub struct BasicBlock {
    pub stmts: Vec<Stmt>,
}

impl BasicBlock {
    pub fn new(stmts: &[Stmt]) -> Self {
        BasicBlock {
            stmts: stmts.to_vec(),
        }
    }

    pub fn get_label(&self) -> usize {
        if let Some(Stmt::Label(l)) = self.stmts.first() {
            *l
        } else {
            0
        }
    }

    pub fn get_goto(&self) -> usize {
        if let Some(Stmt::Jump(j)) = self.stmts.last() {
            j.goto
        } else if let Some(Stmt::CJump(j)) = self.stmts.last() {
            j.goto
        } else {
            0
        }
    }

    pub fn gather_variables_defined(&self) -> Vec<SSA> {
        let mut def: Vec<SSA> = vec![];
        for s in &self.stmts {
            if let Stmt::Tac(tac) = s {
                if let Operand::Assignable(v) = tac.lval {
                    def.push(v);
                }
            }
        }
        def
    }

    pub fn remove_statement(&mut self, stmt: Stmt) {
        let len = self.stmts.len();
        for i in 0..len {
            if self.stmts[i] == stmt {
                self.stmts.remove(i);
                break;
            }
        }
    }

    pub fn patch_phi(&mut self, x: SSA, w: &mut Vec<Stmt>) {
        for s in &mut self.stmts {
            s.patch_phi(x, w);
        }
    }
}
