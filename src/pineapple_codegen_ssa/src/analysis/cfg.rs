use pineapple_data_structures::graph::Graph;
use pineapple_ir::mir::Label;
use pineapple_ir::mir::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::analysis::basic_block::BasicBlock;

use super::basic_block::{BlockEntry, BlockExit};

type Statement = RefCell<Stmt>;
type StatementIndex = usize;
type BlockIndex = usize;

#[derive(Debug)]
pub struct CFG {
    pub entry_label: Label,
    pub blocks: Vec<BasicBlock>,
    pub statements: Vec<Statement>,
    pub graph: Graph,
}

impl From<Vec<Stmt>> for CFG {
    fn from(linear_code: Vec<Stmt>) -> Self {
        let mut entry_label: Option<Label> = None;

        let mut blocks: Vec<BasicBlock> = vec![];
        let mut block: BasicBlock = BasicBlock::new(0);

        let mut statements: Vec<Statement> = vec![];

        let mut block_labels: HashMap<Label, usize> = HashMap::new();
        let mut graph = Graph::default();

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
                _ => {
                    block.statements.push(statements.len());
                    statements.push(RefCell::new(statement))
                }
            }
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

        CFG {
            entry_label: entry_label.unwrap(),
            blocks,
            statements,
            graph,
        }
    }
}
