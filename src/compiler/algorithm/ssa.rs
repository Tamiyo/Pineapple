use crate::compiler::control_flow::basic_block::BasicBlock;
use crate::compiler::control_flow::ControlFlowGraph;
use crate::compiler::three_address::component::*;

use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;

pub fn convert_cfg_to_ssa(cfg: &mut ControlFlowGraph) {
    compute_dominator_tree(cfg);
    compute_immediate_dominators(cfg);
    compute_dominance_frontier(cfg);
    insert_phi_functions(cfg);
    convert_vars_to_ssa(cfg);
    edge_split(cfg);
}
// https://en.wikipedia.org/wiki/Dominator_(graph_theory)
fn compute_dominator_tree(cfg: &mut ControlFlowGraph) {
    // dominator of the start node is the start of the cfg
    cfg.dominator.insert_dominator(0, 0);
    let len = cfg.blocks.len();

    // for all other nodes, set all nodes as the dominators
    for i in 1..len {
        for j in 0..len {
            cfg.dominator.insert_dominator(i, j);
        }
    }

    // iteratively eliminate nodes that are not dominators
    let mut changed = true;
    while changed {
        changed = false;

        for n in 1..len {
            let singleton = HashSet::from_iter(vec![n]);

            let mut diff: HashSet<usize> = HashSet::new();
            for (i, p) in cfg.pred[n].iter().enumerate() {
                if i == 0 {
                    diff = cfg.dominator.get_dominator(*p).clone();
                } else {
                    diff = diff
                        .intersection(cfg.dominator.get_dominator(*p))
                        .copied()
                        .collect();
                }
            }
            let unioned = diff.union(&singleton).copied().collect();
            let prev = cfg.dominator.get_dominator(n).clone();
            cfg.dominator.set_dominator(n, unioned);
            if cfg.dominator.get_dominator(n) != &prev {
                changed = true;
            }
        }
    }
}

// Slide 8 : http://web.cs.iastate.edu/~weile/cs641/4.ControlFlowAnalysis.pdf
fn compute_immediate_dominators(cfg: &mut ControlFlowGraph) {
    for n in 1..cfg.blocks.len() {
        let mut min_index = 0;
        let mut min_value = cfg.blocks.len();
        for e in cfg.dominator.get_dominator(n) {
            if *e != n && n - e < min_value {
                min_value = n - e;
                min_index = *e;
            }
        }
        cfg.dominator.insert_immediate(n, min_index);
    }
}

// https://en.wikipedia.org/wiki/Static_single_assignment_form#Converting_out_of_SSA_form
fn compute_dominance_frontier(cfg: &mut ControlFlowGraph) {
    let len = cfg.blocks.len();
    for b in 0..len {
        if cfg.pred[b].len() >= 2 {
            for p in &cfg.pred[b] {
                let mut runner = *p;
                while runner != cfg.dominator.get_immediate_at(b, 0) {
                    cfg.dominator.insert_frontier(runner, b);
                    runner = cfg.dominator.get_immediate_at(runner, 0);
                }
            }
        }
    }
}

// Appel p. 407
fn insert_phi_functions(cfg: &mut ControlFlowGraph) {
    let mut defsites: HashMap<SSA, HashSet<usize>> = HashMap::new();
    let mut aorig: Vec<HashSet<SSA>> = vec![HashSet::new(); cfg.blocks.len()];
    let mut aphi: Vec<HashSet<SSA>> = vec![HashSet::new(); cfg.blocks.len()];

    for (i, n) in cfg.blocks.iter().enumerate() {
        for a in n.gather_variables_defined() {
            aorig[i].insert(a);
            defsites.entry(a).or_insert_with(HashSet::new).insert(i);
        }
    }

    for a in &cfg.gather_variables() {
        let mut w: Vec<usize> = defsites.get(a).unwrap().iter().copied().collect();

        while !w.is_empty() {
            let n = w.pop().unwrap();
            for y in cfg.dominator.get_frontier(n) {
                if !aphi[*y].contains(a) && a.is_var {
                    let mut phivec: Vec<Operand> = Vec::new();
                    for _ in &cfg.pred[*y] {
                        phivec.push(Operand::Assignable(SSA {
                            value: a.value,
                            ssa: 0,
                            is_var: a.is_var,
                        }));
                    }
                    cfg.blocks[*y].stmts.insert(
                        1,
                        Stmt::Tac(Tac {
                            lval: Operand::Assignable(SSA {
                                value: a.value,
                                ssa: 0,
                                is_var: a.is_var,
                            }),
                            rval: Expr::Phi(phivec),
                        }),
                    );

                    aphi[*y].insert(*a);
                    if !aorig[*y].contains(a) {
                        w.push(*y);
                    }
                }
            }
        }
    }
}

// Appel p. 409
fn convert_vars_to_ssa(cfg: &mut ControlFlowGraph) {
    let mut count: HashMap<(usize, bool), usize> = HashMap::new();
    let mut stack: HashMap<(usize, bool), Vec<usize>> = HashMap::new();
    for a in &cfg.gather_variables() {
        count.insert((a.value, a.is_var), 0);
        stack.insert((a.value, a.is_var), vec![0]);
    }

    rename(cfg, 0, &mut count, &mut stack);
}

// Appel p. 409
fn rename(
    cfg: &mut ControlFlowGraph,
    n: usize,
    count: &mut HashMap<(usize, bool), usize>,
    stack: &mut HashMap<(usize, bool), Vec<usize>>,
) {
    for s in &mut cfg.blocks[n].stmts {
        // if s is not a Φ function, repalce the variables
        for x in s.used() {
            let top = stack.get(&(x.value, x.is_var)).unwrap().last();
            if let Some(i) = top {
                s.replace_use_with_ssa(x.value, x.is_var, *i)
            }
        }

        // update count definition
        for a in s.defined() {
            let i = count.get(&(a.value, a.is_var)).unwrap();
            stack.get_mut(&(a.value, a.is_var)).unwrap().push(*i);
            s.replace_def_with_ssa(a.value, a.is_var, *i);
            *count.get_mut(&(a.value, a.is_var)).unwrap() += 1;
        }
    }

    // update successors
    for y in &cfg.succ[n] {
        let j = cfg.pred[*y].iter().position(|&r| r == n).unwrap();
        for stmt in &mut cfg.blocks[*y].stmts {
            if let Stmt::Tac(tac) = stmt {
                if let Expr::Phi(args) = &tac.rval {
                    let a = match &args[0] {
                        Operand::Assignable(v) => v,
                        _ => panic!("expected assignable"),
                    };
                    if let Some(i) = stack.get(&(a.value, a.is_var)).expect("").last() {
                        tac.rval.replace_phi_def(j, *i)
                    }
                }
            }
        }
    }

    // rename children
    let mut children: Vec<usize> = vec![];
    for (i, domi) in cfg.dominator.immediate.iter().enumerate() {
        if domi.contains(&n) {
            children.push(i);
        }
    }
    for x in children {
        rename(cfg, x, count, stack);
    }

    for s in &cfg.blocks[n].stmts {
        for a in s.defined() {
            stack
                .get_mut(&(a.value, a.is_var))
                .expect("Expect var to exist")
                .pop();
        }
    }
}

// Appel p.408
fn edge_split(cfg: &mut ControlFlowGraph) {
    for a in 0..cfg.blocks.len() {
        for b in cfg.succ[a].clone() {
            if cfg.succ[a].len() > 1 && cfg.pred[b].len() > 1 {
                let pos = cfg.succ[a].iter().position(|&r| r == b).unwrap();
                cfg.succ[a].remove(pos);

                let pos = cfg.pred[b].iter().position(|&r| r == a).unwrap();
                cfg.pred[b].remove(pos);

                let z = cfg.blocks.len();
                cfg.blocks.push(BasicBlock::new(&[]));
                cfg.succ[a].push(z);
                cfg.pred[b].push(z);
            }
        }
    }
}
