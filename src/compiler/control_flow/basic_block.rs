use crate::compiler::three_address_code::Operand;
use crate::compiler::three_address_code::Stmt;
use core::cell::RefCell;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub statements: Vec<Rc<RefCell<Stmt>>>,
}

impl BasicBlock {
    pub fn def(&self) -> Vec<Operand> {
        let mut v = vec![];
        for statement in &self.statements {
            v.append(&mut statement.borrow().vars_defined());
        }
        v
    }

    pub fn used(&self) -> Vec<Operand> {
        let mut v = vec![];
        for statement in &self.statements {
            v.append(&mut statement.borrow().vars_used());
        }
        v
    }

    pub fn get_label(&self) -> usize {
        match *self.statements[0].borrow() {
            Stmt::Label(label) => label,
            _ => 0,
        }
    }

    pub fn get_jump(&self) -> usize {
        match *self.statements[self.statements.len() - 1].borrow() {
            Stmt::Jump(label) | Stmt::CJump(_, label) => label,
            _ => 0,
        }
    }

    pub fn get_variables_defined(&self) -> Vec<Operand> {
        let mut variables: Vec<Operand> = Vec::new();

        for statement in &self.statements {
            variables.append(&mut statement.borrow().vars_defined());
        }

        variables
    }

    pub fn remove_statement_at_index(&mut self, index: usize) {
        self.statements.remove(index);
    }

    pub fn remove_statement(&mut self, stmt: Rc<RefCell<Stmt>>) {
        let len = self.statements.len();
        for i in 0..len {
            if *self.statements[i].borrow() == *stmt.borrow() {
                self.statements.remove(i);
                break;
            }
        }
    }

    pub fn patch_phi(&mut self, x: Operand, w: &mut Vec<Rc<RefCell<Stmt>>>) {
        for s in &mut self.statements {
            let res = match s.try_borrow_mut() {
                Ok(mut r) => r.patch_phi(x),
                _ => false,
            };

            if res {
                w.push(Rc::clone(s));
            }
        }
    }
}
