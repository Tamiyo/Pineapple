use crate::compiler::dominator::DominatorContext;
use crate::compiler::ir::Stmt;
use crate::compiler::ir::{Oper};
use crate::util::graph::{DirectedGraph, Graph};

use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;
use std::{fmt, rc::Rc};

type Statement = Rc<RefCell<Stmt>>;
type BlockIndex = usize;
type StatementIndex = usize;

#[derive(Clone)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    label: Option<Stmt>,
    goto: Option<Stmt>,
}

impl BasicBlock {
    pub fn new() -> Self {
        BasicBlock {
            statements: vec![],
            label: None,
            goto: None,
        }
    }

    pub fn label(&self) -> Option<usize> {
        match self.label {
            Some(Stmt::Label(label)) => Some(label),
            _ => None,
        }
    }

    pub fn jump(&self) -> Option<usize> {
        match self.label {
            Some(Stmt::Jump(label)) | Some(Stmt::CJump(_, label)) => Some(label),
            _ => None,
        }
    }

    pub fn insert_at_beginning(&mut self, statement: Stmt) {
        self.statements.insert(0, Rc::new(RefCell::new(statement)));
    }

    pub fn insert_at_end(&mut self, statement: Stmt) {
        self.statements.push(Rc::new(RefCell::new(statement)));
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut i = 0;
        match &self.label {
            Some(label) => writeln!(f, "{}:\t{:?}", i, label)?,
            None => writeln!(f, "{}:\tNone", i)?,
        };
        i += 1;
        for statement in &self.statements {
            writeln!(f, "{}:\t{:?}", i, *statement.borrow())?;
            i += 1;
        }
        match &self.goto {
            Some(goto) => writeln!(f, "{}:\t{:?}", i, goto)?,
            None => writeln!(f, "{}:\tNone", i)?,
        };
        writeln!(f)?;
        i += 1;
        Ok(())
    }
}

// Maybe abstract out the analysis sets from the cfg object?
// Dominators -> its own thing
// Dataflow analysis -> its own thing
pub struct CFG {
    // CFG
    pub blocks: Vec<BasicBlock>,
    pub graph: DirectedGraph<usize>,

    pub uses: HashMap<Oper, HashSet<StatementIndex>>,
    pub def: HashMap<Oper, HashSet<StatementIndex>>,

    // Dominators
    pub dom_ctx: DominatorContext
    ,
    // Dataflow Analysis
    // http://www.cs.cmu.edu/afs/cs/academic/class/15745-s06/web/handouts/04.pdf
}

impl CFG {
    // https://ethz.ch/content/dam/ethz/special-interest/infk/inst-cs/lst-dam/documents/Education/Classes/Spring2016/2810_Advanced_Compiler_Design/Homework/slides_hw1.pdf
    pub fn statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];

        for bb in &self.blocks {
            match &bb.label() {
                Some(label) => statements.push(Rc::new(RefCell::new(Stmt::Label(*label)))),
                _ => (),
            }
            for statement in &bb.statements {
                statements.push(Rc::clone(statement));
            }
            match &bb.goto {
                Some(stmt) => statements.push(Rc::new(RefCell::new(stmt.clone()))),
                _ => (),
            }
        }

        statements
    }

    pub fn replace_all_operand_with(&mut self, orig: &Oper, new: &Oper) {
        for bb in &mut self.blocks {
            for stmt in &bb.statements {
                stmt.borrow_mut().replace_all_oper_def_with(orig, new);
                stmt.borrow_mut().replace_all_oper_use_with(orig, new);
            }
            
            match &mut bb.goto {
                Some(stmt) => {
                    stmt.replace_all_oper_use_with(orig, new);
                }
                _ => (),
            }
        }
    }
}

// Ah... this monster method to convert tac to a cfg.
// This is my 5th iteration of this method and I still dont like it.
// Dont even try to read it just know it works, probably.
impl From<&Vec<Stmt>> for CFG {
    fn from(linear_code: &Vec<Stmt>) -> Self {
        let mut label_count = 0;
        let mut blocks: Vec<BasicBlock> = vec![];
        let mut current_block_statements: Vec<Statement> = vec![];

        let mut blabel: Option<Stmt> = None;
        let mut bgoto: Option<Stmt> = None;

        let mut uses: HashMap<Oper, HashSet<usize>> = HashMap::new();
        let mut def: HashMap<Oper, HashSet<usize>> = HashMap::new();

        for statement in linear_code {
            match statement {
                Stmt::Label(label) => {
                    label_count = max(label_count, *label + 1);
                    blabel = Some(statement.clone());
                }
                Stmt::Jump(_) | Stmt::CJump(_, _) => {
                    bgoto = Some(statement.clone());

                    let block = BasicBlock {
                        statements: current_block_statements.clone(),
                        label: blabel,
                        goto: bgoto,
                    };

                    blocks.push(block);
                    blabel = None;
                    bgoto = None;
                    current_block_statements.clear();
                }
                Stmt::Tac(lval, rval) => {
                    let nindex = blocks.len();
                    def.entry(*lval).or_insert_with(HashSet::new).insert(nindex);

                    for oper in rval.oper_used() {
                        uses.entry(oper).or_insert_with(HashSet::new).insert(nindex);
                    }
                    current_block_statements.push(Rc::new(RefCell::new(statement.clone())));
                }
                _ => {
                    current_block_statements.push(Rc::new(RefCell::new(statement.clone())));
                }
            };
        }

        blocks.push(BasicBlock {
            statements: current_block_statements,
            label: blabel,
            goto: bgoto,
        });

        let mut graph: DirectedGraph<usize> = DirectedGraph::new();
        // Maps labels back to their respective blocks
        let mut label_to_block: Vec<usize> = vec![0; blocks.len()];

        for i in 0..blocks.len() {
            graph.insert(i);
            label_to_block[blocks[i].label().unwrap()] = i;
        }

        for b in 0..blocks.len() {
            let goto = &blocks[b].goto;

            match goto {
                Some(Stmt::CJump(_, label)) => {
                    let block_containing_label = label_to_block[*label];
                    graph.create_edge(b, block_containing_label);
                    graph.create_edge(b, b + 1);
                }
                Some(Stmt::Jump(label)) => {
                    let block_containing_label = label_to_block[*label];
                    graph.create_edge(b, block_containing_label);
                }
                _ => (),
            }
        }

        CFG {
            blocks,
            graph,
            dom_ctx: DominatorContext::default(),
            uses,
            def,
        }
    }
}

impl fmt::Display for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut i = 0;
        for node in &self.blocks {
            match &node.label {
                Some(label) => writeln!(f, "{}:\t{:?}", i, label)?,
                None => writeln!(f, "{}:\tNone", i)?,
            };
            i += 1;
            for statement in &node.statements {
                writeln!(f, "{}:\t\t{:?}", i, *statement.borrow())?;
                i += 1;
            }
            match &node.goto {
                Some(goto) => writeln!(f, "{}:\t\t{:?}", i, goto)?,
                None => writeln!(f, "{}:\t\tgoto _PGRM_END", i)?,
            };
            writeln!(f)?;
            i += 1;
        }
        Ok(())
    }
}
