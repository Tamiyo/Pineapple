use crate::compiler::dominator::DominatorContext;
use crate::compiler::ir::Oper;
use crate::compiler::ir::{Expr, Stmt};
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
    pub label: Option<Statement>,
    pub goto: Option<Statement>,
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
        match &self.label {
            // NamedLabel could cause problems
            Some(reference) => match &*reference.borrow() {
                Stmt::Label(label) | Stmt::NamedLabel(label) => Some(*label),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn jump(&self) -> Option<usize> {
        match &self.goto {
            Some(reference) => match &*reference.borrow() {
                Stmt::Jump(label) | Stmt::CJump(_, label) => Some(*label),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn oper_defined(&self) -> Vec<Oper> {
        let mut variables: Vec<Oper> = Vec::new();

        for statement in &self.statements {
            variables.append(&mut statement.borrow().def());
        }

        variables
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
            Some(label) => writeln!(f, "{}:\t{:?}", i, *label.borrow())?,
            None => writeln!(f, "{}:\tNone", i)?,
        };
        i += 1;
        for statement in &self.statements {
            writeln!(f, "{}:\t{:?}", i, *statement.borrow())?;
            i += 1;
        }
        match &self.goto {
            Some(goto) => writeln!(f, "{}:\t{:?}", i, *goto.borrow())?,
            None => writeln!(f, "{}:\tNone", i)?,
        };
        writeln!(f)?;
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

    pub stack_offset: usize,

    // Dominators
    pub dom_ctx: DominatorContext,
    // Dataflow Analysis
    // http://www.cs.cmu.edu/afs/cs/academic/class/15745-s06/web/handouts/04.pdf
}

impl CFG {
    // https://ethz.ch/content/dam/ethz/special-interest/infk/inst-cs/lst-dam/documents/Education/Classes/Spring2016/2810_Advanced_Compiler_Design/Homework/slides_hw1.pdf
    pub fn statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];

        for bb in 0..self.blocks.len() {
            let bb = &self.blocks[bb];
            if !(bb.label == None && bb.goto == None && bb.statements.is_empty()) {
                match &bb.label {
                    Some(label) => statements.push(Rc::clone(label)),
                    _ => (),
                }
                for statement in &bb.statements {
                    statements.push(Rc::clone(statement));
                }

                if let Some(stmt) = &bb.goto {
                    statements.push(Rc::clone(stmt));
                }
            }
        }

        statements
    }

    pub fn statements_using(&mut self, v: &Oper) -> Vec<Rc<RefCell<Stmt>>> {
        let mut statements: Vec<Rc<RefCell<Stmt>>> = vec![];

        for block in &self.blocks {
            for statement in &block.statements {
                if statement.borrow().used().contains(v) {
                    statements.push(Rc::clone(statement));
                }
            }

            if let Some(statement) = &block.goto {
                if statement.borrow().used().contains(v) {
                    statements.push(Rc::clone(statement));
                }
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

            if let Some(stmt) = &mut bb.goto {
                stmt.borrow_mut().replace_all_oper_use_with(orig, new);
            }
        }
    }

    pub fn remove_statement(&mut self, s: Rc<RefCell<Stmt>>) {
        for block in &mut self.blocks {
            let len = block.statements.len();
            for i in 0..len {
                if block.statements[i] == s {
                    block.statements.remove(i);
                    break;
                }
            }
        }
    }

    pub fn remove_conditional_jump(
        &mut self,
        stmt: &Statement,
        res: bool,
        label: usize,
    ) -> (Vec<Statement>, Option<usize>) {
        // Find the block that the cjump belongs to
        let block = self
            .blocks
            .iter()
            .position(|b| match &b.goto {
                Some(s) => s == stmt,
                None => false,
            })
            .unwrap();

        // if res is true then we jump, so we delete the fallthrough
        let mut modified_statements = vec![];
        if res {
            modified_statements.append(&mut self.remove_block(block + 1));
        }
        // if res is false we fallthrough, so delete the other solution
        else {
            let block_to_remove = self
                .blocks
                .iter()
                .position(|b| match b.jump() {
                    Some(l) => {
                        if l == label {
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                })
                .unwrap();
                modified_statements.append(&mut self.remove_block(block_to_remove));
        };

        // Replace the statement with a straight jump
        let jump_label = match &self.blocks[block].goto {
            Some(stmt) => {
                if let Stmt::CJump(_, label) = &*stmt.borrow() {
                    Some(*label)
                } else {
                    None
                }
            }
            _ => None,
        };

        (modified_statements, jump_label)
    }

    fn remove_block(&mut self, block_to_remove: usize) -> Vec<Statement> {
        if block_to_remove == 0 || self.graph.pred[block_to_remove].len() > 0 {
            return vec![];
        }

        // patch phi functions
        // for block in &mut self.blocks {
        //     for s in 0..block.statements.len() {
        //         let statement = block.statements[s].clone();
        //         if let Stmt::Tac(lval, Expr::Phi(args)) = &mut *statement.borrow_mut() {
        //             for i in 0..args.len() {
        //                 if args[i].1 == block_to_remove {
        //                     args.remove(i);

        //                     // Check to see if we can simplify the phi function
        //                     if args.len() == 0 {
        //                         block.statements.remove(s);
        //                     } else if args.len() == 1 {
        //                         statement.replace(Stmt::Tac(*lval, Expr::Oper(args[0].0)));
        //                     }
        //                     break;
        //                 }
        //             }
        //         };
        //     }
        // }

        // actual removal
        let mut modified_statements = vec![];

        self.blocks[block_to_remove].label = None;
        self.blocks[block_to_remove].goto = None;
        modified_statements.append(
            &mut self.blocks[block_to_remove]
                .statements
                .iter()
                .map(|s| Rc::clone(s))
                .collect(),
        );

        let children = self.graph.succ[block_to_remove].clone();
        self.graph.remove(&block_to_remove);

        // Recursively delete any blocks that no long have and preds
        for child in children {
            println!("child: {:?}", child);
            println!("pred[child]: {:?}", self.graph.pred[child]);
            if self.graph.pred[child].is_empty() {
                modified_statements.append(&mut self.remove_block(child));
            }
        }

        modified_statements
    }
}

// Ah... this monster method to convert tac to a cfg.
// This is my 5th iteration of this method and I still dont like it.
// Dont even try to read it just know it works, probably.
impl From<&(Vec<Stmt>, usize)> for CFG {
    fn from(transformed_code: &(Vec<Stmt>, usize)) -> Self {
        let (linear_code, stack_offset) = transformed_code;

        let mut label_count = 0;
        let mut blocks: Vec<BasicBlock> = vec![];
        let mut current_block_statements: Vec<Statement> = vec![];

        let mut blabel: Option<Statement> = None;
        let mut bgoto: Option<Statement> = None;

        let mut uses: HashMap<Oper, HashSet<usize>> = HashMap::new();
        let mut def: HashMap<Oper, HashSet<usize>> = HashMap::new();

        for statement in linear_code {
            match statement {
                Stmt::Label(label) => {
                    label_count = max(label_count, *label + 1);
                    blabel = Some(Rc::new(RefCell::new(statement.clone())));
                }
                Stmt::NamedLabel(_) => {
                    blabel = Some(Rc::new(RefCell::new(statement.clone())));
                }
                Stmt::Jump(_) | Stmt::Call(_, _) | Stmt::CJump(_, _) => {
                    bgoto = Some(Rc::new(RefCell::new(statement.clone())));

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
        let mut label_to_block: HashMap<usize, usize> = HashMap::new();

        for (i, _) in blocks.iter().enumerate() {
            graph.insert(i);
            if let Some(b) = blocks[i].label() {
                label_to_block.insert(b, i);
            }
        }

        for (b, _) in blocks.iter().enumerate() {
            let goto = &blocks[b].goto;

            match goto {
                Some(reference) => match &*reference.borrow() {
                    Stmt::CJump(_, label) => {
                        let block_containing_label = label_to_block.get(label).unwrap();
                        graph.create_edge(b, *block_containing_label);
                        graph.create_edge(b, b + 1);
                    }
                    Stmt::Jump(label) => {
                        let block_containing_label = label_to_block.get(label).unwrap();
                        graph.create_edge(b, *block_containing_label);
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        CFG {
            blocks,
            graph,
            dom_ctx: DominatorContext::default(),
            uses,
            def,
            stack_offset: *stack_offset,
        }
    }
}

impl fmt::Debug for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut i = 0;
        for node in &self.blocks {
            if !(node.label == None && node.goto == None && node.statements.is_empty()) {
                match &node.label {
                    Some(label) => writeln!(f, "{}:\t{:?}", i, *label.borrow())?,
                    None => writeln!(f, "{}:\tNone", i)?,
                };
                i += 1;
                for statement in &node.statements {
                    writeln!(f, "{}:\t\t{:?}", i, *statement.borrow())?;
                    i += 1;
                }
                match &node.goto {
                    Some(goto) => writeln!(f, "{}:\t\t{:?}", i, *goto.borrow())?,
                    None => writeln!(f)?,
                };
                writeln!(f)?;
                i += 1;
            }
        }
        Ok(())
    }
}
