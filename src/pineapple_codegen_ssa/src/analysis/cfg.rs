use pineapple_data_structures::graph::Graph;
use pineapple_ir::mir::Stmt;
use pineapple_ir::mir::{Label, Oper};
use std::{cell::RefCell, collections::HashSet};
use std::{collections::HashMap, rc::Rc};

use crate::analysis::basic_block::BasicBlock;

use super::{
    basic_block::{BlockEntry, BlockExit},
    dominator::DominatorContext,
};

type Statement = Rc<RefCell<Stmt>>;
type StatementIndex = usize;
type BlockIndex = usize;

#[derive(Clone)]
pub struct CFG {
    pub entry_label: Label,
    pub blocks: Vec<BasicBlock>,

    pub defined: HashMap<Oper, HashSet<StatementIndex>>,
    pub used: HashMap<Oper, HashSet<StatementIndex>>,

    pub graph: Graph,
    pub dominator: DominatorContext,
}

impl CFG {
    pub fn active_statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];
        for block in &self.blocks {
            for s in &block.statements {
                statements.push(Rc::clone(s));
            }
        }
        statements
    }

    pub fn remove_statement(&mut self, statement: Statement) {
        for block in &mut self.blocks {
            for i in 0..block.statements.len() {
                if block.statements[i] == statement {
                    block.statements.remove(i);
                    break;
                }
            }
        }
    }

    pub fn get_statements_using_oper(&mut self, oper: &Oper) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];

        for block in &self.blocks {
            for statement in &block.statements {
                if statement.borrow().oper_used().contains(oper) {
                    statements.push(Rc::clone(statement));
                }
            }

            if let BlockExit::Exit(statement) = &block.exit {
                if statement.borrow().oper_used().contains(oper) {
                    statements.push(Rc::clone(statement));
                }
            }
        }

        statements
    }

    pub fn replace_all_operand_with(&mut self, orig: &Oper, new: &Oper) {
        for block in &self.blocks {
            for stmt in &block.statements {
                stmt.borrow_mut().replace_all_oper_def_with(orig, new);
                stmt.borrow_mut().replace_all_oper_use_with(orig, new);
            }

            if let BlockExit::Exit(statement) = &block.exit {
                statement.borrow_mut().replace_all_oper_use_with(orig, new);
            }
        }
    }
}

impl From<Vec<Stmt>> for CFG {
    fn from(linear_code: Vec<Stmt>) -> Self {
        let mut entry_label: Option<Label> = None;

        let mut blocks: Vec<BasicBlock> = vec![];
        let mut block: BasicBlock = BasicBlock::new(0);

        let mut block_labels: HashMap<Label, usize> = HashMap::new();
        let mut graph = Graph::default();

        let mut defined: HashMap<Oper, HashSet<StatementIndex>> = HashMap::new();
        let mut used: HashMap<Oper, HashSet<StatementIndex>> = HashMap::new();

        for statement in linear_code {
            match statement {
                Stmt::Label(label) => {
                    graph.add_node();

                    if entry_label == None {
                        entry_label = Some(label);
                    }

                    let statement = Rc::new(RefCell::new(statement));

                    if block.entry == BlockEntry::None {
                        block.entry = BlockEntry::Entry(statement);
                    }
                    block_labels.insert(label, blocks.len());
                }
                Stmt::Jump(_) => {
                    let statement = Rc::new(RefCell::new(statement));
                    block.exit = BlockExit::Exit(statement);

                    blocks.push(block);
                    block = BasicBlock::new(blocks.len());
                }
                Stmt::CJump(_, _) => {
                    let statement = Rc::new(RefCell::new(statement));
                    block.exit = BlockExit::Exit(statement);

                    blocks.push(block);
                    block = BasicBlock::new(blocks.len());
                }
                Stmt::Tac(lval, ref rval) => {
                    defined
                        .entry(lval)
                        .or_insert_with(HashSet::new)
                        .insert(blocks.len());

                    for oper in rval.oper_used() {
                        used.entry(oper)
                            .or_insert_with(HashSet::new)
                            .insert(blocks.len());
                    }
                    let statement = Rc::new(RefCell::new(statement));
                    block.statements.push(statement);
                }
                _ => {
                    let statement = Rc::new(RefCell::new(statement));
                    block.statements.push(statement);
                }
            }
        }

        if !block.statements.is_empty() {
            blocks.push(block);
        }

        for block in &blocks {
            if let BlockExit::Exit(exit) = &block.exit {
                match &*exit.borrow() {
                    Stmt::Jump(label) => {
                        let jumpto_block_index = *block_labels.get(&label).unwrap();
                        graph.add_directed_edge(block.index, jumpto_block_index);
                    }
                    Stmt::CJump(_, label) => {
                        let jumpto_block_index = *block_labels.get(&label).unwrap();
                        graph.add_directed_edge(block.index, block.index + 1);
                        graph.add_directed_edge(block.index, jumpto_block_index);
                    }
                    _ => (),
                };
            }
        }

        let mut cfg = CFG {
            entry_label: entry_label.unwrap(),
            blocks,
            graph,
            defined,
            used,
            dominator: DominatorContext::default(),
        };
        crate::analysis::dominator::compute_dominator_context(&mut cfg);
        cfg
    }
}

impl std::fmt::Debug for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.blocks {
            match &block.entry {
                BlockEntry::Entry(s) => {
                    write!(f, "{:?}\n", &*s.borrow())?;
                }
                BlockEntry::None => {}
            }
            for s in &block.statements {
                write!(f, "\t{:?}\n", &*s.borrow())?;
            }
            match &block.exit {
                BlockExit::Exit(s) => {
                    write!(f, "\t{:?}\n", &*s.borrow())?;
                }
                BlockExit::None => {}
            }
            println!();
        }
        Ok(())
    }
}
