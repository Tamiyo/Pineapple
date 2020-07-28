use crate::compiler::control_flow::basic_block::BasicBlock;
use crate::compiler::dominator::Dominator;
use crate::compiler::three_address_code::{Expr, Operand, Stmt};
use crate::graph::DirectedGraph;

use std::fmt;

pub mod basic_block;

pub struct ControlFlowContext {
    pub cfg: ControlFlowGraph,
    pub dominator: Dominator,
}

impl ControlFlowContext {
    pub fn new(tacs: Vec<Stmt>) -> Self {
        let cfg = ControlFlowGraph::new(tacs);
        let size = cfg.blocks.len();

        ControlFlowContext {
            cfg,
            dominator: Dominator::new(size),
        }
    }
}

impl fmt::Debug for ControlFlowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "GRAPH:\n{:?}DOMINATOR TREE:\n{:?}",
            self.cfg, self.dominator
        )
    }
}

pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub graph: DirectedGraph<usize>,
}

impl ControlFlowGraph {
    pub fn new(tacs: Vec<Stmt>) -> Self {
        let mut blocks: Vec<BasicBlock> = vec![];
        let mut start: usize = 0;
        let mut label_count = 1;
        for (i, stmt) in tacs.iter().enumerate() {
            match stmt {
                Stmt::Label(_) => {
                    if i - start != 0 {
                        let mut block = BasicBlock::new(&tacs[start..i]);
                        match block.statements.last() {
                            Some(Stmt::Jump(_)) | Some(Stmt::CJump(_, _)) => (),
                            _ => {
                                let next_label = &tacs[i];
                                if let Stmt::Label(l) = next_label {
                                    block.statements.push(Stmt::Jump(*l));
                                }
                            }
                        }
                        blocks.push(block);
                        start = i;
                    }
                }
                Stmt::Jump(_) | Stmt::CJump(_, _) => {
                    let mut block = BasicBlock::new(&tacs[start..i + 1]);
                    if let Some(Stmt::Label(_)) = block.statements.first() {
                    } else {
                        let label = tacs.len() + label_count;
                        label_count += 1;
                        block.statements.insert(0, Stmt::Label(label));
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

        // Connect Nodes
        let mut graph: DirectedGraph<usize> = DirectedGraph::new();
        let mut label_to_node: Vec<Vec<usize>> = vec![vec![]; tacs.len() + label_count];
        for (i, block) in blocks.iter().enumerate() {
            graph.insert(i);
            label_to_node[block.get_label()].push(i);
        }

        for i in 0..blocks.len() {
            let g = blocks[i].get_goto();
            for n in &label_to_node[g] {
                if i < blocks.len() - 1 && *n > 0 {
                    graph.add_edge(i, *n);
                }
            }
        }
        // Fix offset if statements
        for i in 1..blocks.len() {
            if graph.pred[&i].is_empty() {
                graph.add_edge(i - 1, i);
            }
        }

        ControlFlowGraph {
            blocks: blocks,
            graph: graph,
        }
    }

    pub fn get_variables(&self) -> Vec<Operand> {
        let mut vars: Vec<Operand> = vec![];

        for (_, b) in self.blocks.iter().enumerate() {
            for s in &b.statements {
                vars.append(&mut s.vars_defined());
            }
        }
        vars
    }

    /*
        This is a somewhat lazy implementation, I would like to remove this
        soon. All this does is allow me to lazily implement some features, and
        it is very inefficient as it does a linear scan / collection  each call.

        Maybe some form of "CFG_CONTEXT" to keep track of specific attributes like
        statements, locations, etc.
    */
    pub fn get_statements(&self) -> Vec<Stmt> {
        let mut stmts: Vec<Stmt> = vec![];

        for (i, b) in self.blocks.iter().enumerate() {
            for s in &b.statements {
                stmts.push(s.clone());
            }
        }

        stmts
    }

    pub fn get_statements_using(&mut self, v: Operand) -> Vec<&mut Stmt> {
        let mut stmts: Vec<&mut Stmt> = vec![];

        for (_, b) in self.blocks.iter_mut().enumerate() {
            for s in &mut b.statements {
                if s.uses_var(v) {
                    stmts.push(s);
                }
            }
        }
        stmts
    }

    pub fn replace_statement_rval_with(&mut self, stmt: Stmt, rval: Operand) -> Stmt {
        for (_, b) in self.blocks.iter_mut().enumerate() {
            for s in &mut b.statements {
                if stmt == *s {
                    if let Stmt::Tac(_, r) = s {
                        *r = Expr::Operand(rval);
                    }
                    return s.clone();
                }
            }
        }
        stmt
    }

    pub fn replace_all_operand_with(&mut self, orig: Operand, new: Operand) {
        for (_, b) in self.blocks.iter_mut().enumerate() {
            for s in &mut b.statements {
                s.replace_all_operand_with(orig, new);
            }
        }
    }

    fn remove_block(&mut self, b: usize, w: &mut Vec<Stmt>) {
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

    /*
        Like "get_statements()", this is pretty inefficient bc it does a
        linear scan every call.  It might be "better" to just keep track
        of variable locations (look at this later).
    */
    pub fn remove_statement(&mut self, stmt: Stmt) {
        let len = self.blocks.len();
        for i in 0..len {
            let slen = self.blocks[i].statements.len();
            for j in 0..slen {
                if self.blocks[i].statements[j] == stmt {
                    self.blocks[i].remove_statement(&stmt);
                    return;
                }
            }
        }
    }

    pub fn remove_conditional_jump(&mut self, stmt: Stmt, condition: bool, w: &mut Vec<Stmt>) {
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
            let goto_label = self.blocks[i].get_goto();
            for (bi, b) in &mut self.blocks.iter().enumerate() {
                if b.get_label() == goto_label {
                    self.remove_block(bi, w);
                    break;
                }
            }
        }

        self.remove_statement(stmt);
    }
}

impl fmt::Debug for ControlFlowGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        for (i, b) in self.blocks.iter().enumerate() {
            if b.statements.len() > 2 {
                write!(f, "BLOCK {:?}\n{:#?}\n\n", i, b.statements)?;
            }
        }
        write!(f, "DAG: {:?}\n\n", self.graph)
    }
}
