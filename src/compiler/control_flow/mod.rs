// pub mod component;
use crate::compiler::control_flow::basic_block::BasicBlock;
use crate::compiler::control_flow::dominator::Dominator;
use crate::compiler::three_address::component::*;
use std::collections::HashSet;

use std::fmt;

pub mod basic_block;
pub mod dominator;

pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub exec: Vec<bool>,

    pub pred: Vec<Vec<usize>>,
    pub succ: Vec<Vec<usize>>,

    pub variables: HashSet<SSA>,

    pub dominator: Dominator,
}

impl ControlFlowGraph {
    // See Appel p. 170
    pub fn new(tacs: Vec<Stmt>) -> Self {
        let mut blocks: Vec<BasicBlock> = vec![];
        let mut start: usize = 0;
        let mut label_count = 1;
        for (i, stmt) in tacs.iter().enumerate() {
            match stmt {
                Stmt::Label(_) => {
                    if i - start != 0 {
                        let mut block = BasicBlock::new(&tacs[start..i]);
                        match block.stmts.last() {
                            Some(Stmt::Jump(_)) | Some(Stmt::CJump(_)) => (),
                            _ => {
                                let next_label = &tacs[i];
                                if let Stmt::Label(l) = next_label {
                                    block.stmts.push(Stmt::Jump(Jump { goto: *l }));
                                }
                            }
                        }
                        blocks.push(block);
                        start = i;
                    }
                }
                Stmt::Jump(_) | Stmt::CJump(_) => {
                    let mut block = BasicBlock::new(&tacs[start..i + 1]);
                    if let Some(Stmt::Label(_)) = block.stmts.first() {
                    } else {
                        let label = tacs.len() + label_count;
                        label_count += 1;
                        block.stmts.insert(0, Stmt::Label(Label::Label(label)));
                    }
                    blocks.push(block);
                    start = i + 1;
                }
                _ => (),
            };
        }

        if start != tacs.len() {
            let block = BasicBlock::new(&tacs[start..]);
            blocks.push(block);
        }

        // Connect the nodes
        let mut pred: Vec<Vec<usize>> = vec![vec![]; blocks.len()];
        let mut succ: Vec<Vec<usize>> = vec![vec![]; blocks.len()];
        let mut label_to_node: Vec<Vec<usize>> = vec![vec![]; tacs.len() + label_count];
        for (i, block) in blocks.iter().enumerate() {
            label_to_node[block.get_label()].push(i);
        }

        for i in 0..blocks.len() {
            let g = blocks[i].get_goto();
            for n in &label_to_node[g] {
                if i < blocks.len() - 1 {
                    succ[i].push(*n);
                }
                if *n > 0 {
                    pred[*n].push(i);
                }
            }
        }
        // Fix offset if statements
        for i in 1..blocks.len() {
            if pred[i].is_empty() {
                pred[i].push(i - 1);
                succ[i - 1].push(i);
            }
        }

        let size = blocks.len();
        ControlFlowGraph {
            blocks: blocks,
            exec: vec![true; size],
            pred: pred,
            succ: succ,
            variables: HashSet::new(),
            dominator: Dominator::new(size),
        }
    }

    pub fn gather_statements(&self) -> Vec<Stmt> {
        let mut stmts: Vec<Stmt> = vec![];

        for (i, b) in self.blocks.iter().enumerate() {
            if (self.exec[i] && b.stmts.len() > 2) || i == 0 {
                for s in &b.stmts {
                    stmts.push(s.clone());
                }
            }
        }

        stmts
    }

    pub fn gather_statements_using(&mut self, v: Operand) -> Vec<&mut Stmt> {
        let mut stmts: Vec<&mut Stmt> = vec![];

        for (i, b) in self.blocks.iter_mut().enumerate() {
            if self.exec[i] {
                for s in &mut b.stmts {
                    if s.uses(v) {
                        stmts.push(s);
                    }
                }
            }
        }
        stmts
    }

    pub fn replace_statement_rval_with(&mut self, stmt: Stmt, rval: Operand) -> Stmt {
        for (i, b) in self.blocks.iter_mut().enumerate() {
            if self.exec[i] {
                for s in &mut b.stmts {
                    if stmt == *s {
                        s.replace_rval_with(rval);
                        return s.clone();
                    }
                }
            }
        }

        stmt
    }

    pub fn gather_variables(&self) -> Vec<SSA> {
        let mut vars: Vec<SSA> = vec![];

        for (i, b) in self.blocks.iter().enumerate() {
            if self.exec[i] {
                for s in &b.stmts {
                    vars.append(&mut s.defined());
                }
            }
        }
        vars
    }

    pub fn remove_conditional_jump(&mut self, stmt: Stmt, condition: bool, w: &mut Vec<Stmt>) {
        let mut i = 0;

        for (bi, b) in self.blocks.iter().enumerate() {
            if self.exec[bi] {
                for s in &b.stmts {
                    if stmt == *s {
                        i = bi;
                        break;
                    }
                }
            }
        }

        if condition {
            self.remove_block(i + 1, w);
        } else {
            let goto_label = self.blocks[i].get_goto();
            for (bi, b) in &mut self.blocks.iter().enumerate() {
                if self.exec[bi] && b.get_label() == goto_label {
                    self.remove_block(bi, w);
                    break;
                }
            }
        }

        self.remove_statement(stmt);
    }

    fn remove_block(&mut self, b: usize, w: &mut Vec<Stmt>) {
        for var in self.blocks[b].gather_variables_defined() {
            self.patch_phi(var, w);
        }

        for pred in &self.pred[b] {
            let index = self.succ[*pred].iter().position(|&r| r == b).unwrap();
            self.succ[*pred].remove(index);
        }

        for succ in &self.succ[b].clone() {
            let index = self.pred[*succ].iter().position(|&r| r == b).unwrap();
            self.pred[*succ].remove(index);
            if self.pred[*succ].is_empty() {
                self.remove_block(*succ, w);
            }
        }
        self.exec[b] = false;
        self.pred[b].clear();
        self.succ[b].clear();
    }

    pub fn remove_statement(&mut self, stmt: Stmt) {
        let len = self.blocks.len();
        for i in 0..len {
            let slen = self.blocks[i].stmts.len();
            for j in 0..slen {
                if self.exec[i] && self.blocks[i].stmts[j] == stmt {
                    self.blocks[i].remove_statement(stmt);
                    return;
                }
            }
        }
    }

    fn patch_phi(&mut self, x: SSA, w: &mut Vec<Stmt>) {
        for (bi, b) in self.blocks.iter_mut().enumerate() {
            if self.exec[bi] {
                b.patch_phi(x, w);
            }
        }
    }
}

impl fmt::Debug for ControlFlowGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        for (i, b) in self.blocks.iter().enumerate() {
            if self.exec[i] {
                write!(
                    f,
                    "BLOCK {:?}\npred[n]: {:?}\nsucc[n]: {:?}\n{:#?}\n\n",
                    i, self.pred[i], self.succ[i], b.stmts
                )?;
            }
        }
        Ok(())
    }
}
