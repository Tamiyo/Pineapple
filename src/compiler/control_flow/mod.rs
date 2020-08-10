use crate::compiler::control_flow::basic_block::BasicBlock;
use crate::compiler::dominator::Dominator;
use crate::compiler::three_address_code::Operand;
use crate::compiler::three_address_code::Stmt;
use crate::graph::DirectedGraph;
use std::cell::RefCell;

use std::cmp::max;
use std::{fmt, rc::Rc};

pub mod basic_block;

pub struct ControlFlowContext {
    pub cfg: ControlFlowGraph,
    pub dom: Dominator,
}

impl ControlFlowContext {
    pub fn new(tacs: Vec<Stmt>) -> Self {
        let cfg = ControlFlowGraph::new(tacs);
        let size = cfg.blocks.len();

        ControlFlowContext {
            cfg,
            dom: Dominator::new(size),
        }
    }
}

pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub statements: Vec<Rc<RefCell<Stmt>>>,
    pub graph: DirectedGraph<usize>,
    pub orig_size: usize,
}

impl ControlFlowGraph {
    pub fn new(tacs: Vec<Stmt>) -> Self {
        let mut label_count = 0;

        let mut blocks: Vec<BasicBlock> = vec![];

        let mut block_statements: Vec<Rc<RefCell<Stmt>>> = vec![];
        let mut statements: Vec<Rc<RefCell<Stmt>>> = vec![];

        for stmt in &tacs {
            let statement = Rc::new(RefCell::new(stmt.clone()));
            block_statements.push(Rc::clone(&statement));
            statements.push(statement);

            match stmt {
                Stmt::Label(label) => {
                    label_count = max(label_count, *label + 1);
                }
                Stmt::Jump(_) | Stmt::CJump(_, _) => {
                    let block = BasicBlock {
                        statements: block_statements.clone(),
                    };
                    blocks.push(block);
                    block_statements.clear();
                }
                _ => (),
            };
        }

        if !block_statements.is_empty() {
            let block = BasicBlock {
                statements: block_statements,
            };
            blocks.push(block);
        }

        let mut graph: DirectedGraph<usize> = DirectedGraph::new();
        let mut label_to_block: Vec<Vec<usize>> = vec![vec![]; label_count];

        for i in 0..blocks.len() {
            graph.insert(i);
            label_to_block[blocks[i].get_label()].push(i);
        }

        for i in 0..blocks.len() {
            let g = blocks[i].get_jump();
            for n in &label_to_block[g] {
                if i < blocks.len() - 1 && *n > 0 {
                    graph.add_edge(i, *n);
                }
            }
        }

        for i in 1..blocks.len() {
            if graph.pred[&i].is_empty() {
                graph.add_edge(i - 1, i);
            }
        }

        let orig_size = blocks.len();

        ControlFlowGraph {
            blocks,
            statements,
            orig_size,
            graph,
        }
    }

    pub fn get_statements(&self) -> Vec<Rc<RefCell<Stmt>>> {
        let mut statements: Vec<Rc<RefCell<Stmt>>> = vec![];
        for block in &self.blocks {
            for statement in &block.statements {
                statements.push(Rc::clone(statement));
            }
        }
        statements
    }

    pub fn get_statements_using(&mut self, v: Operand) -> Vec<Rc<RefCell<Stmt>>> {
        let mut statements: Vec<Rc<RefCell<Stmt>>> = vec![];

        for block in &self.blocks {
            for statement in &block.statements {
                if statement.borrow().uses_var(v) {
                    statements.push(Rc::clone(statement));
                }
            }
        }

        statements
    }

    pub fn get_statements_using_with_indices(&self, v: Operand) -> Vec<(usize, usize)> {
        let mut statements: Vec<(usize, usize)> = vec![];

        let mut i = 0;
        for block in &self.blocks {
            let mut j = 0;
            for statement in &block.statements {
                if statement.borrow().uses_var(v) {
                    statements.push((i, j));
                }
                j += 1;
            }
            i += 1;
        }

        statements
    }

    pub fn get_variables(&self) -> Vec<Operand> {
        let mut variables: Vec<Operand> = vec![];

        for block in &self.blocks {
            for statement in &block.statements {
                variables.append(&mut statement.borrow().vars_defined());
            }
        }

        variables
    }

    pub fn replace_all_operand_with(&mut self, orig: Operand, new: Operand) {
        for block in &mut self.blocks {
            for statement in &mut block.statements {
                statement.borrow_mut().replace_all_operand_with(orig, new)
            }
        }
    }

    pub fn replace_all_operand_use_with(&mut self, orig: Operand, new: Operand) {
        for block in &mut self.blocks {
            for statement in &mut block.statements {
                statement
                    .borrow_mut()
                    .replace_all_operand_use_with(orig, new)
            }
        }
    }

    pub fn remove_statement(&mut self, s: Rc<RefCell<Stmt>>) {
        for block in &mut self.blocks {
            let len = block.statements.len();
            for i in 0..len {
                if block.statements[i] == s {
                    block.remove_statement_at_index(i);
                    break;
                }
            }
        }
    }

    pub fn remove_conditional_jump(
        &mut self,
        stmt: Rc<RefCell<Stmt>>,
        condition: bool,
        w: &mut Vec<Rc<RefCell<Stmt>>>,
    ) {
        let mut i = 0;

        for (bi, b) in self.blocks.iter().enumerate() {
            for s in &b.statements {
                if stmt == *s {
                    i = bi;
                    break;
                }
            }
        }

        if condition {
            self.remove_block(i + 1, w);
        } else {
            let goto_label = self.blocks[i].get_jump();
            for (bi, b) in &mut self.blocks.iter().enumerate() {
                if b.get_label() == goto_label {
                    self.remove_block(bi, w);
                    break;
                }
            }
        }

        self.remove_statement(stmt);
    }

    fn remove_block(&mut self, b: usize, w: &mut Vec<Rc<RefCell<Stmt>>>) {
        for var in self.blocks[b].get_variables_defined() {
            for (_, b) in self.blocks.iter_mut().enumerate() {
                b.patch_phi(var, w);
            }
        }

        for succ in self.graph.succ[&b].clone() {
            match self.graph.pred.get_mut(&succ) {
                Some(set) => {
                    set.remove(&b);
                    if self.graph.pred[&succ].is_empty() {
                        self.remove_block(succ, w);
                    }
                }
                _ => (),
            };
        }

        self.blocks[b].statements.clear();
        self.graph.remove(b);
    }
}

impl fmt::Debug for ControlFlowGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut i = 0;
        for block in &self.blocks {
            if block.statements.len() > 0 {
                for statement in &block.statements {
                    write!(f, "{}:\t{:?}\n", i, *statement.borrow())?;
                    i += 1;
                }
                write!(f, "\n")?;
            }
        }
        write!(f, "{:?}", self.graph)?;
        Ok(())
    }
}
