use std::{cell::RefCell, collections::HashMap, collections::HashSet, rc::Rc};

use indexmap::IndexSet;
use pineapple_ir::mir::{Expr, Oper, Stmt, SSA};

use crate::analysis::cfg::CFG;

pub fn construct_ssa(cfg: &mut CFG) {
    insert_phi_functions(cfg);
    rename_variables(cfg);
}

pub fn destruct_ssa(cfg: &mut CFG) {
    convert_to_conventional_ssa(cfg);
    rename_variables(cfg);
    remove_phi_functions(cfg);
    sequence_parallel_copies(cfg);
    flatten_parallel_copies(cfg);
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Variable {
    Variable(usize),
    Temp(usize),
}

fn insert_phi_functions(cfg: &mut CFG) {
    for (v, _) in &cfg.defined {
        let mut f: HashSet<usize> = HashSet::new();
        let mut w: IndexSet<usize> = IndexSet::new();

        if let Oper::SSA(SSA::Var(_, _)) = v {
            for d in &cfg.defined[v] {
                w.insert(*d);
            }
        }

        while !w.is_empty() {
            let x = w.pop().unwrap();
            for y in &cfg.dominator.domf[x] {
                if !f.contains(&y) {
                    let phivec: Vec<(Oper, usize)> =
                        vec![(*v, 0); cfg.graph.predecessors(*y).len()];

                    let statement = Rc::new(RefCell::new(Stmt::Tac(*v, Expr::Phi(phivec))));
                    cfg.blocks[*y].statements.insert(0, statement);

                    f.insert(*y);
                    if !cfg.defined[v].contains(&y) {
                        w.insert(*y);
                    }
                }
            }
        }
    }
}

fn rename_variables(cfg: &mut CFG) {
    let mut count: HashMap<Variable, usize> = HashMap::new();
    let mut stack: HashMap<Variable, Vec<usize>> = HashMap::new();

    for bb in &cfg.blocks {
        for statement in &bb.statements {
            for oper in &mut statement.borrow_mut().oper_defined() {
                match oper {
                    Oper::SSA(SSA::Var(value, ssa)) => {
                        *ssa = 0;
                        count.insert(Variable::Variable(*value), 0);
                        stack.insert(Variable::Variable(*value), vec![0]);
                    }
                    Oper::SSA(SSA::Temp(value, ssa)) => {
                        *ssa = 0;
                        // count.insert(Variable::Temp(*value), 0);
                        // stack.insert(Variable::Temp(*value), vec![0]);
                    }
                    _ => (),
                }
            }
        }
    }

    rename(cfg, 0, &mut count, &mut stack);
}

fn rename(
    cfg: &mut CFG,
    n: usize,
    count: &mut HashMap<Variable, usize>,
    stack: &mut HashMap<Variable, Vec<usize>>,
) {
    for statement in &cfg.blocks[n].statements {
        let statement = &mut *statement.borrow_mut();
        if let Stmt::Tac(_, Expr::Phi(_)) = statement {
        } else {
            for x in statement.oper_used() {
                if let Oper::SSA(SSA::Var(_, _)) = x {
                    let x_min = match x {
                        Oper::SSA(SSA::Var(value, _)) => Variable::Variable(value),
                        // Oper::SSA(SSA::Temp(value, _)) => Variable::Temp(value),
                        _ => panic!(format!("expected variable. got {:?} instead", x)),
                    };
                    let i = match stack.get(&x_min) {
                        Some(s) => s.last().unwrap(),
                        None => panic!("expected variable"),
                    };
                    statement.replace_oper_use_with_ssa(x, *i);
                }
            }
        }

        for a in statement.oper_defined() {
            if let Oper::SSA(SSA::Var(_, _)) = a {
                let a_min = match a {
                    Oper::SSA(SSA::Var(value, _)) => Variable::Variable(value),
                    // Oper::SSA(SSA::Temp(value, _)) => Variable::Temp(value),
                    _ => panic!("expected variable"),
                };
                let c = count[&a_min];
                count.insert(a_min, c + 1);

                let i = c;
                stack.get_mut(&a_min).unwrap().push(i);
                statement.replace_oper_def_with_ssa(a, i);
            }
        }
    }

    for y in cfg.graph.successors(n).iter() {
        let j = cfg
            .graph
            .predecessors(*y)
            .iter()
            .position(|x| x == &n)
            .unwrap();
        for statement in &cfg.blocks[*y].statements {
            if let Stmt::Tac(_, Expr::Phi(args)) = &mut *statement.borrow_mut() {
                let a = args.get_mut(j).unwrap();
                let a0_min = match a.0 {
                    Oper::SSA(SSA::Var(value, _)) => Variable::Variable(value),
                    // Oper::SSA(SSA::Temp(value, _)) => Variable::Temp(value),
                    _ => panic!("expected variable"),
                };

                let i = stack[&a0_min].last().unwrap();
                a.0.replace_with_ssa(a.0, *i);
                a.1 = n;
            }
        }
    }

    for (x, domi) in cfg.dominator.idom.clone().iter().enumerate() {
        if let Some(domi) = domi {
            if *domi == n {
                rename(cfg, x, count, stack);
            }
        }
    }

    for statement in &cfg.blocks[n].statements {
        for a in statement.borrow().oper_defined() {
            if let Oper::SSA(SSA::Var(_, _)) = a {
                let a_min = match a {
                    Oper::SSA(SSA::Var(value, _)) => Variable::Variable(value),
                    Oper::SSA(SSA::Temp(value, _)) => Variable::Temp(value),
                    _ => panic!("expected variable"),
                };
                stack
                    .get_mut(&a_min)
                    .unwrap_or_else(|| panic!("tried unwrapping for: {:?}", a))
                    .pop();
            }
        }
    }
}

/* TODO
    Why would the index be greater than the len if we're explicity adding to the end of this?
        This has to be something wrong with the parallel copy code...
*/
fn convert_to_conventional_ssa(cfg: &mut CFG) {
    for bb in &mut cfg.blocks {
        let statement = Stmt::ParallelCopy(vec![]);
        bb.statements.insert(0, Rc::new(RefCell::new(statement)));

        let statement = Stmt::ParallelCopy(vec![]);
        bb.statements.push(Rc::new(RefCell::new(statement)));
    }

    for b0_ind in 0..cfg.blocks.len() {
        let b0 = &cfg.blocks[b0_ind];
        for statement in b0.statements.iter() {
            if let Stmt::Tac(a0, Expr::Phi(args)) = &mut *statement.borrow_mut() {
                for (ai, bi) in args.iter_mut() {
                    let pci = cfg.blocks[*bi].statements.last().unwrap();

                    let ai_prime = match ai {
                        Oper::SSA(SSA::Var(value, _)) => Oper::SSA(SSA::Var(*value, 0)),
                        // Oper::SSA(SSA::Temp(value, _)) => Oper::SSA(SSA::Temp(*value, 0)),
                        _ => panic!("Expected var in phi function"),
                    };

                    match &mut *pci.borrow_mut() {
                        Stmt::ParallelCopy(pcopy) => {
                            let statement =
                                Rc::new(RefCell::new(Stmt::Tac(ai_prime, Expr::Oper(*ai))));
                            pcopy.push(statement);
                        }
                        _ => panic!(""),
                    }
                    *ai = ai_prime;
                }

                // begin
                let pc0 = b0.statements.first().unwrap();
                let a0_prime = match a0 {
                    Oper::SSA(SSA::Var(value, _)) => Oper::SSA(SSA::Var(*value, 0)),
                    Oper::SSA(SSA::Temp(value, _)) => Oper::SSA(SSA::Temp(*value, 0)),
                    _ => panic!("Expected var in phi function"),
                };

                match &mut *pc0.borrow_mut() {
                    Stmt::ParallelCopy(pcopy) => {
                        let statement = Rc::new(RefCell::new(Stmt::Tac(*a0, Expr::Oper(a0_prime))));
                        pcopy.push(statement);
                    }
                    _ => panic!("expected pc0 to be a parallel copy"),
                }
                *a0 = a0_prime;
            };
        }
    }
}

fn remove_phi_functions(cfg: &mut CFG) {
    for b0_ind in 0..cfg.blocks.len() {
        let b0 = &cfg.blocks[b0_ind];
        let mut to_remove = vec![];

        for (index, statement) in b0.statements.clone().iter().enumerate() {
            if let Stmt::Tac(a0, Expr::Phi(args)) = &mut *statement.borrow_mut() {
                for (ai, bi) in args {
                    let statement = Rc::new(RefCell::new(Stmt::Tac(*a0, Expr::Oper(*ai))));
                    cfg.blocks[*bi].statements.push(statement);
                }

                // remove_phi_func
                to_remove.push(index);
            }
        }

        // Again I REALLY dont like using these giant iter.filter.map.collects but
        // it is 1am. Sleepy Tamiyo writes bad code that works and is easy
        // so here we are. Will do something about this in the future.
        cfg.blocks[b0_ind].statements = cfg.blocks[b0_ind]
            .statements
            .iter()
            .enumerate()
            .filter(|(i, _)| !to_remove.contains(i))
            .map(|(_, v)| v.clone())
            .collect();
    }
}

fn sequence_parallel_copies(cfg: &mut CFG) {
    for bb in &cfg.blocks {
        for stmt in &bb.statements {
            if let Stmt::ParallelCopy(pcopy) = &mut *stmt.borrow_mut() {
                sequence_parallel_copy(pcopy);
            }
        }
    }
}

// Holy **** the code below is AWFUL, I HATE EVERY LINE OF IT.... BUT IT WORKS FOR NOW WILL FIX LATER
fn sequence_parallel_copy(pcopy: &mut Vec<Rc<RefCell<Stmt>>>) {
    let mut seq: Vec<Rc<RefCell<Stmt>>> = vec![];

    loop {
        // Conditional checking
        let same = pcopy
            .iter()
            .filter(|stmt| {
                if let Stmt::Tac(lval, Expr::Oper(rval)) = *stmt.borrow() {
                    lval == rval
                } else {
                    false
                }
            })
            .count();

        if pcopy.is_empty() || same != pcopy.len() {
            break;
        }

        // algo
        pcopy.clone().iter().enumerate().all(|(i, stmt1)| {
            if let Stmt::Tac(b1, Expr::Oper(a)) = &mut *stmt1.borrow_mut() {
                let cond = pcopy.iter().find(|stmt2| match &stmt2.try_borrow() {
                    Ok(refer) => {
                        if let Stmt::Tac(c, Expr::Oper(b2)) = **refer {
                            if a != &c && b1 == &b2 {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => false,
                });

                if cond == None {
                    seq.push(Rc::clone(stmt1));
                    pcopy.remove(i);
                    false
                } else if a != b1 {
                    let aprime = match a {
                        Oper::SSA(SSA::Var(sym, _)) => Oper::SSA(SSA::Var(*sym, 0)),
                        _ => panic!(""),
                    };
                    let copy = Stmt::Tac(aprime, Expr::Oper(*a));
                    seq.push(Rc::new(RefCell::new(copy)));
                    *a = aprime;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        });
    }
    if !seq.is_empty() {
        *pcopy = seq;
    }
}

fn flatten_parallel_copies(cfg: &mut CFG) {
    for b in 0..cfg.blocks.len() {
        let mut s = 0;
        loop {
            if s >= cfg.blocks[b].statements.len() {
                break;
            }
            if let Stmt::ParallelCopy(copies) = &*cfg.blocks[b].statements[s].clone().borrow() {
                for copy in copies.iter() {
                    cfg.blocks[b].statements.insert(s + 1, Rc::clone(copy));
                }
                cfg.blocks[b].statements.remove(s);
            } else {
                s += 1;
            }
        }
    }
}
