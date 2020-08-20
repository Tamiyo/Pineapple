use crate::compiler::ir::Stmt;
use crate::compiler::ir::{Expr, Oper};
use crate::util::graph::{DirectedGraph, Graph};
use indexmap::IndexSet;
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
    statements: Vec<Statement>,
}

impl BasicBlock {
    pub fn new() -> Self {
        BasicBlock { statements: vec![] }
    }

    pub fn label(&self) -> usize {
        match *self.statements[0].borrow() {
            Stmt::Label(label) => label,
            _ => 0,
        }
    }

    pub fn jump(&self) -> usize {
        match *self.statements[self.statements.len() - 1].borrow() {
            Stmt::Jump(label) | Stmt::CJump(_, label) => label,
            _ => 0,
        }
    }

    pub fn insert_at_beginning(&mut self, statement: Stmt) {
        self.statements.insert(0, Rc::new(RefCell::new(statement)));
    }
}

// Maybe abstract out the analysis sets from the cfg object?
// Dominators -> its own thing
// Dataflow analysis -> its own thing
pub struct CFG {
    // CFG
    blocks: Vec<BasicBlock>,
    graph: DirectedGraph<usize>,

    // Dominators
    dom: Vec<HashSet<usize>>,
    idom: Vec<IndexSet<usize>>,
    domf: Vec<HashSet<usize>>,

    // Dataflow Analysis
    gen: HashMap<StatementIndex, HashSet<BlockIndex>>,
    kill: HashMap<StatementIndex, HashSet<BlockIndex>>,

    uses: HashMap<Oper, HashSet<StatementIndex>>,
    def: HashMap<Oper, HashSet<StatementIndex>>,
    // http://www.cs.cmu.edu/afs/cs/academic/class/15745-s06/web/handouts/04.pdf
}

impl CFG {
    pub fn new() -> Self {
        CFG {
            blocks: vec![],
            graph: DirectedGraph::new(),
            dom: vec![],
            idom: vec![],
            domf: vec![],
            gen: HashMap::new(),
            kill: HashMap::new(),
            uses: HashMap::new(),
            def: HashMap::new(),
        }
    }

    // General
    pub fn generate_from_linear_code(&mut self, linear_code: &[Stmt]) {
        // For Dataflow Analysis
        let mut uses: HashMap<Oper, HashSet<usize>> = HashMap::new();
        let mut def: HashMap<Oper, HashSet<usize>> = HashMap::new();

        // For CFG Creation
        let mut label_count = 0;
        let mut blocks: Vec<BasicBlock> = vec![];
        let mut current_block_statements: Vec<Statement> = vec![];

        for statement in linear_code {
            current_block_statements.push(Rc::new(RefCell::new(statement.clone())));

            match statement {
                Stmt::Label(label) => {
                    label_count = max(label_count, *label + 1);
                }
                Stmt::Jump(_) | Stmt::CJump(_, _) => {
                    let block = BasicBlock {
                        statements: current_block_statements.clone(),
                    };
                    blocks.push(block);
                    current_block_statements.clear();
                }
                Stmt::Tac(lval, rval) => {
                    let nindex = blocks.len();
                    def.entry(*lval).or_insert_with(HashSet::new).insert(nindex);

                    for oper in rval.oper_used() {
                        uses.entry(oper).or_insert_with(HashSet::new).insert(nindex);
                    }
                }
                _ => (),
            };
        }

        if !current_block_statements.is_empty() {
            let block = BasicBlock {
                statements: current_block_statements,
            };
            blocks.push(block);
        }

        blocks
            .last_mut()
            .unwrap()
            .statements
            .push(Rc::new(RefCell::new(Stmt::Jump(label_count))));

        blocks.push(BasicBlock {
            statements: vec![Rc::new(RefCell::new(Stmt::Label(label_count)))],
        });

        let mut graph: DirectedGraph<usize> = DirectedGraph::new();
        let mut label_to_block: Vec<Vec<usize>> = vec![vec![]; label_count + 2];

        for i in 0..blocks.len() {
            graph.insert(i);
            label_to_block[blocks[i].label()].push(i);
        }

        for i in 0..blocks.len() {
            let g = blocks[i].jump();
            for n in &label_to_block[g] {
                if i < blocks.len() - 1 && *n > 0 {
                    graph.create_edge(i, *n);
                }
            }
        }

        for i in 1..blocks.len() {
            if graph.pred[&i].is_empty() {
                graph.create_edge(i - 1, i);
            }
        }

        let size = blocks.len();
        self.blocks = blocks;
        self.graph = graph;
        self.dom = vec![HashSet::new(); size];
        self.idom = vec![IndexSet::new(); size];
        self.domf = vec![HashSet::new(); size];
        self.uses = uses;
        self.def = def;
    }

    pub fn statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = vec![];

        for bb in &self.blocks {
            for statement in &bb.statements {
                statements.push(Rc::clone(statement));
            }
        }

        statements
    }

    // SSA Form
    pub fn insert_phi_functions(&mut self) {
        for (v, _) in self.def.iter() {
            let mut f: HashSet<usize> = HashSet::new();
            let mut w: IndexSet<usize> = IndexSet::new();

            /*
                [TODO]
                SSA BOOK :- p.31
                Does this apply to only variables, or does a temporary count as a variable as well?
            */
            for d in &self.def[v] {
                // if let Oper::Var(_, _) = v {
                //     w.insert(*d);
                // }
                w.insert(*d);
            }

            while !w.is_empty() {
                let x = w.pop().unwrap();
                for y in &self.domf[x] {
                    if !f.contains(y) {
                        let mut phivec: Vec<Oper> = Vec::new();

                        for _ in &self.graph.pred[y] {
                            phivec.push(*v);
                        }

                        let statement = Stmt::Tac(*v, Expr::Phi(phivec));
                        self.blocks[*y].insert_at_beginning(statement);

                        f.insert(*y);
                        if !self.def[v].contains(y) {
                            w.insert(*y);
                        }
                    }
                }
            }
        }
    }

    pub fn construct_ssa(&mut self) {
        let mut count: HashMap<(usize, usize), usize> = HashMap::new();
        let mut stack: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

        // Do we only want vars or are temporaries OK when it comes to the use-def chains?
        for a in self.def.keys() {
            count.insert(a.as_non_ssa(), 0);
            stack.insert(a.as_non_ssa(), Vec::new());
            stack.get_mut(&a.as_non_ssa()).unwrap().push(0);
        }

        self.rename(0, &mut count, &mut stack);
    }

    // The problem here is that we're hashing on SSA values which we dont want for this stage...
    // https://gist.github.com/CMCDragonkai/2f4b5e078f690443d190
    fn rename(&mut self, n: usize, count: &mut HashMap<(usize, usize), usize>, stack: &mut HashMap<(usize, usize), Vec<usize>>) {
        for s in &mut self.blocks[n].statements {
            let s = &mut *s.borrow_mut();
            if let Stmt::Tac(_, Expr::Phi(_)) = s {} else {
                for x in s.used() {
                    let i = stack[&x.as_non_ssa()].last().unwrap();
                    s.replace_oper_use_with_ssa(x, *i);
                }
            }

            for a in s.def() {
                let c = count[&a.as_non_ssa()];
                count.insert(a.as_non_ssa(), c + 1);

                let i = c + 1;
                stack.get_mut(&a.as_non_ssa()).unwrap().push(i);
                s.replace_oper_def_with_ssa(a, i);
            }
        }

        for (j, y) in self.graph.succ[&n].iter().enumerate() {
            for s in &self.blocks[*y].statements {
                let s = &mut *s.borrow_mut();
                if let Stmt::Tac(_, Expr::Phi(args)) = s {
                    let a = args.get_mut(j).unwrap();
                    let i = stack[&a.as_non_ssa()].last().unwrap();
                    a.replace_with_ssa(*a, *i);
                }
            }
        }

        for (x, domi) in self.idom.clone().iter().enumerate() {
            if domi.contains(&n) {
                self.rename(x, count, stack);
            }
        }

        for s in &self.blocks[n].statements {
            for a in s.borrow().def() {
                stack
                    .get_mut(&a.as_non_ssa())
                    .unwrap_or_else(|| panic!("tried unwrapping for: {:?}", a))
                    .pop();
            }
        }
    }

    // Idk if we need this yet
    fn phiweb_discovery(&mut self) {
        let mut phiweb: HashMap<Oper, HashSet<Oper>> = HashMap::new();

        for v in self.def.keys() {
            let mut set: HashSet<Oper> = HashSet::new();
            set.insert(*v);
            phiweb.insert(*v, set);
        }

        for bb in &self.blocks {
            for instruction in &bb.statements {
                if let Stmt::Tac(lval, Expr::Phi(args)) = &*instruction.borrow() {
                    for ai in args {
                        let ai_web = phiweb.get(ai).unwrap().clone();
                        let dest_web = phiweb.get_mut(lval).unwrap();
                        dest_web.extend(ai_web.iter());
                    }
                }
            }
        }
    }

    fn as_conventional_ssa(&mut self) {
        for b in 0..self.blocks.len() {
            self.blocks[b]
                .statements
                .insert(0, Rc::new(RefCell::new(Stmt::ParallelCopy)));
            self.blocks[b]
                .statements
                .push(Rc::new(RefCell::new(Stmt::ParallelCopy)));
        }

        for b in &self.blocks {
            for s in &b.statements {
                if let Stmt::Tac(lval, Expr::Phi(args)) = &mut *s.borrow_mut() {
                    for a in args {
                        let pc1 = b.statements.last().unwrap();

                        let a1_prime = match a {
                            Oper::Var(value, _) => Oper::Var(*value, 0),
                            Oper::Temp(value, _) => Oper::Temp(*value, 0),
                            _ => panic!("Expected var in phi function"),
                        };
                        pc1.replace(Stmt::Tac(a1_prime, Expr::Oper(*a)));
                        *a = a1_prime;
                    }

                    let pc0 = b.statements.first().unwrap();

                    let a0_prime = match lval {
                        Oper::Var(value, _) => Oper::Var(*value, 0),
                        Oper::Temp(value, _) => Oper::Temp(*value, 0),
                        _ => panic!("Expected var in phi function"),
                    };
                    println!("{:?} = {:?}", pc0, Stmt::Tac(a0_prime, Expr::Oper(*lval)));
                    pc0.replace(Stmt::Tac(a0_prime, Expr::Oper(*lval)));
                    *lval = a0_prime;
                }
            }
        }
    }

    fn remove_empty_parallel_copies(&mut self) {
        for b in 0..self.blocks.len() {
            if let Stmt::ParallelCopy = *self.blocks[b].statements.first().unwrap().clone().borrow()
            {
                self.blocks[b].statements.remove(0);
            }

            if let Stmt::ParallelCopy = *self.blocks[b].statements.last().unwrap().clone().borrow()
            {
                let last = self.blocks[b].statements.len() - 1;
                self.blocks[b].statements.remove(last);
            }
        }
    }

    pub fn destruct_ssa(&mut self) {
        self.as_conventional_ssa();
        self.remove_empty_parallel_copies(); // We could move this to deadcode elim?
    }

    // Dominators
    pub fn compute_dom(&mut self) {
        // dominator of the start node is the start of the cfg
        self.dom[0].insert(0);
        let len = self.blocks.len();

        // for all other blocks, set all blocks as the dominators
        for i in 1..len {
            for j in 0..len {
                self.dom[i].insert(j);
            }
        }

        // iteratively eliminate blocks that are not dominators
        let mut changed = true;
        while changed {
            changed = false;

            for n in 1..len {
                let singleton: HashSet<usize> = vec![n].into_iter().collect();

                let mut diff: HashSet<usize> = HashSet::new();
                for (i, p) in self.graph.pred.get(&n).unwrap().iter().enumerate() {
                    if i == 0 {
                        diff = self.dom[*p].clone();
                    } else {
                        diff = diff.intersection(&self.dom[*p]).copied().collect();
                    }
                }
                let unioned = diff.union(&singleton).copied().collect();
                let prev = self.dom[n].clone();
                self.dom[n] = unioned;
                if self.dom[n] != prev {
                    changed = true;
                }
            }
        }
    }

    pub fn compute_idom(&mut self) {
        for n in 1..self.blocks.len() {
            let mut min_index = 0;
            let mut min_value = self.blocks.len();
            for e in &self.dom[n] {
                if *e != n && n - e < min_value {
                    min_value = n - e;
                    min_index = *e;
                }
            }
            self.idom[n].insert(min_index);
        }
    }

    pub fn compute_domf(&mut self) {
        let len = self.blocks.len();
        for b in 0..len {
            if self.graph.pred.get(&b).unwrap().len() >= 2 {
                for p in self.graph.pred.get(&b).unwrap().iter() {
                    let mut runner = *p;
                    while runner != self.idom[b][0] {
                        self.domf[runner].insert(b);
                        runner = self.idom[runner][0];
                    }
                }
            }
        }
    }

    // Dataflow Analysis
    // pub fn compute_gen_and_kill(&mut self) {
    //     // http://www.cs.columbia.edu/~suman/secure_sw_devel/Basic_Program_Analysis_DF.pdf

    //     let mut gen_statement: HashMap<usize, HashSet<usize>> = HashMap::new();
    //     let mut kill_statement: HashMap<usize, HashSet<usize>> = HashMap::new();

    //     let mut i = 0;
    //     let mut b = 0;
    //     for node in &self.blocks {
    //         gen_node.insert(b, HashSet::new());
    //         kill_node.insert(b, HashSet::new());
    //         for statement in &node.statements {
    //             gen_statement.insert(i, HashSet::new());
    //             match *statement.borrow() { 
    //                 Stmt::Tac(lval, Expr::Oper(_)) => {
    //                     gen_statement.get_mut(&i).unwrap().insert(i);

    //                     let mut kill_set: HashSet<usize> = vec![i].into_iter().collect();
    //                     kill_set = self.def[&lval].difference(&kill_set).copied().collect();

    //                     kill_statement.insert(i, kill_set);
    //                 }
    //                 _ => (),
    //             };
    //             i += 1;
    //         }
    //         b += 1;
    //     }

    //     // self.gen_statement = gen_statement;
    //     // self.kill_statement = kill_statement;
    // }

    // pub fn compute_reaching_definition(&mut self) {
    //     let mut reach_in: HashMap<usize, HashSet<usize>> = HashMap::new();
    //     let mut reach_out: HashMap<usize, HashSet<usize>> = HashMap::new();

    //     for n in 0..self.blocks.len() {
    //         // reach_out.insert(n, HashSet::new());
    //         reach_out.insert(n, self.gen[&n].clone());
    //     }

    //     let mut changed: IndexSet<usize> = (0..self.blocks.len()).collect();

    //     while !changed.is_empty() {
    //         let n = changed.pop().unwrap();

    //         reach_in.insert(n, HashSet::new());

    //         for p in &self.graph.pred[&n] {
    //             reach_in.insert(n, reach_in[&n].union(&reach_out[&p]).cloned().collect());
    //         }

    //         let oldout = reach_out[&n].clone();

    //         let diff: HashSet<usize> = reach_in[&n].difference(&self.kill[&n]).cloned().collect();
    //         reach_out.insert(n, self.gen[&n].union(&diff).cloned().collect());

    //         if reach_out[&n] != oldout {
    //             for s in &self.graph.succ[&n] {
    //                 changed.insert(*s);
    //             }
    //         }
    //     }
    // }
}

impl fmt::Debug for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let mut i = 0;
        for node in &self.blocks {
            if !node.statements.is_empty() {
                for statement in &node.statements {
                    write!(f, "{}:\t{:?}\n", i, *statement.borrow())?;
                    i += 1;
                }
                write!(f, "\n")?;
            }
        }
        write!(
            f,
            "::DOMINATORS::\n\tDOM[]: {:?}\n\tIDOM[]: {:?}\n\tDOMF[]: {:?}\n\n",
            self.dom, self.idom, self.domf
        )?;
        write!(
            f,
            "::ANALYSIS::\n\tGEN[]: {:?}\n\tKILL[]: {:?}\n\tUSE[]: {:?}\n\tDEF[]: {:?}\n",
            self.gen, self.kill, self.uses, self.def
        )?;
        Ok(())
    }
}
