use pineapple_data_structures::graph::Graph;
use pineapple_ir::mir::Stmt;
use pineapple_ir::mir::{Label, Oper};
use std::collections::HashMap;
use std::{cell::RefCell, collections::HashSet};

use crate::analysis::basic_block::BasicBlock;

use super::{
    basic_block::{BlockEntry, BlockExit},
    dominator::DominatorContext,
};

type Statement = RefCell<Stmt>;
type StatementIndex = usize;
type BlockIndex = usize;

#[derive(Debug)]
pub struct CFG {
    pub entry_label: Label,
    pub blocks: Vec<BasicBlock>,
    pub statements: Vec<Statement>,

    pub defined: HashMap<Oper, HashSet<StatementIndex>>,
    pub used: HashMap<Oper, HashSet<StatementIndex>>,

    pub graph: Graph,
    pub dominator: DominatorContext,
}

impl CFG {
    pub fn replace_all_operand_with(&mut self, orig: &Oper, new: &Oper) {
        // This could be optimizated by looping over the ENTRY and EXIT of a block instead of going
        // over ALL statements
        for stmt in &self.statements {
            stmt.borrow_mut().replace_all_oper_def_with(orig, new);
            stmt.borrow_mut().replace_all_oper_use_with(orig, new);
        }
    }
}

impl From<Vec<Stmt>> for CFG {
    fn from(linear_code: Vec<Stmt>) -> Self {
        let mut entry_label: Option<Label> = None;

        let mut blocks: Vec<BasicBlock> = vec![];
        let mut block: BasicBlock = BasicBlock::new(0);

        let mut statements: Vec<Statement> = vec![];

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

                    if block.entry == BlockEntry::None {
                        block.entry = BlockEntry::Entry(statements.len());
                    }
                    block_labels.insert(label, blocks.len());
                    statements.push(RefCell::new(statement))
                }
                Stmt::Jump(_) => {
                    block.exit = BlockExit::Exit(statements.len());
                    statements.push(RefCell::new(statement));

                    blocks.push(block);

                    block = BasicBlock::new(blocks.len());
                }
                Stmt::CJump(_, _) => {
                    block.exit = BlockExit::Exit(statements.len());
                    statements.push(RefCell::new(statement));

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
                    block.statements.push(statements.len());
                    statements.push(RefCell::new(statement));
                }
                _ => {
                    block.statements.push(statements.len());
                    statements.push(RefCell::new(statement));
                }
            }
        }

        if !block.statements.is_empty() {
            blocks.push(block);
        }

        for block in &blocks {
            if let BlockExit::Exit(exit) = block.exit {
                match &*statements[exit].borrow() {
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
            statements,
            graph,
            defined,
            used,
            dominator: DominatorContext::default(),
        };
        crate::analysis::dominator::compute_dominator_context(&mut cfg);
        cfg
    }
}
