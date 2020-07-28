use crate::compiler::three_address_code::{Expr, Operand, Stmt};

pub struct BasicBlock {
    pub statements: Vec<Stmt>,
}

impl BasicBlock {
    pub fn new(statements: &[Stmt]) -> Self {
        BasicBlock {
            statements: statements.to_vec(),
        }
    }

    pub fn get_label(&self) -> usize {
        if let Some(Stmt::Label(l)) = self.statements.first() {
            *l
        } else {
            0
        }
    }

    pub fn get_goto(&self) -> usize {
        if let Some(Stmt::Jump(j)) = self.statements.last() {
            *j
        } else if let Some(Stmt::CJump(_, j)) = self.statements.last() {
            *j
        } else {
            0
        }
    }

    pub fn get_variables_defined(&self) -> Vec<Operand> {
        let mut def: Vec<Operand> = vec![];
        for s in &self.statements {
            if let Stmt::Tac(lval, _) = s {
                if let Operand::Assignable(_, _, _) = lval {
                    def.push(*lval);
                }
            }
        }
        def
    }

    pub fn remove_statement(&mut self, stmt: &Stmt) {
        let len = self.statements.len();
        for i in 0..len {
            if self.statements[i] == *stmt {
                if let Stmt::Tac(_, Expr::StackPop) = self.statements[i] {
                    self.statements[i] = Stmt::StackPop;
                } else {
                    self.statements.remove(i);
                }
                break;
            }
        }
    }

    pub fn patch_phi(&mut self, x: Operand, w: &mut Vec<Stmt>) {
        for s in &mut self.statements {
            s.patch_phi(x, w);
        }
    }
}
