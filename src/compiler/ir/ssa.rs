use super::{Expr, Stmt};
use crate::compiler::flowgraph::cfg::CFG;
use crate::compiler::ir::Oper;
use core::cell::RefCell;
use indexmap::IndexSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

fn insert_phi_functions(cfg: &mut CFG) {
    for (v, _) in cfg.def.iter() {
        let mut f: HashSet<usize> = HashSet::new();
        let mut w: IndexSet<usize> = IndexSet::new();

        /*
            [TODO]
            SSA BOOK :- p.31
            Does this apply to only variables, or does a temporary count as a variable as well?
            8/30/2020 - Literature is unclear, proceeding with only variables for now...
        */
        if let Oper::Var(_, _) = v {
            for d in &cfg.def[v] {
                w.insert(*d);
            }
        }
        // for d in &cfg.def[v] {
        //     w.insert(*d);
        // }

        while !w.is_empty() {
            let x = w.pop().unwrap();
            for y in &cfg.dom_ctx.domf[x] {
                if !f.contains(&y) {
                    let mut phivec: Vec<(Oper, usize)> = Vec::new();

                    for _ in &cfg.graph.pred[y] {
                        phivec.push((*v, 0));
                    }

                    let statement = Stmt::Tac(*v, Expr::Phi(phivec));
                    cfg.blocks[*y].insert_at_beginning(statement);

                    f.insert(*y);
                    if !cfg.def[v].contains(y) {
                        w.insert(*y);
                    }
                }
            }
        }
    }
}

fn rename_variables(cfg: &mut CFG) {
    let mut count: HashMap<(usize, bool), usize> = HashMap::new();
    let mut stack: HashMap<(usize, bool), Vec<usize>> = HashMap::new();

    // Do we only want vars or are temporaries OK when it comes to the use-def chains?
    // for a in cfg.def.keys() {
    //     if let Oper::Var(_, ssa) = a {
    //         count.insert(a.as_non_ssa(), 0);
    //         stack.insert(a.as_non_ssa(), vec![0]);
    //     }
    //     //     count.insert(a.as_non_ssa(), 0);
    //     //     stack.insert(a.as_non_ssa(), vec![0]);
    // }

    for bb in &cfg.blocks {
        for statement in &bb.statements {
            for oper in &mut statement.borrow_mut().def() {
                match oper {
                    Oper::Var(_, ssa) => {
                        *ssa = 0;
                        count.insert(oper.as_non_ssa(), 0);
                        stack.insert(oper.as_non_ssa(), vec![0]);
                    }
                    Oper::Temp(_, ssa) => {
                        *ssa = 0;
                    }
                    _ => (),
                }
            }
        }
    }

    rename(cfg, 0, &mut count, &mut stack);
}

// https://gist.github.com/CMCDragonkai/2f4b5e078f690443d190
// This is a inefficient version of the stack-based ssa renaming algo. For now this
// will suffice but in the future this should be reworked.
fn rename(
    cfg: &mut CFG,
    n: usize,
    count: &mut HashMap<(usize, bool), usize>,
    stack: &mut HashMap<(usize, bool), Vec<usize>>,
) {
    for s in &mut cfg.blocks[n].statements {
        let s = &mut *s.borrow_mut();
        if let Stmt::Tac(_, Expr::Phi(_)) = s {
        } else {
            for x in s.used() {
                if let Oper::Var(_, _) = x {
                    let i = stack[&x.as_non_ssa()].last().unwrap();
                    s.replace_oper_use_with_ssa(x, *i);
                }
                // let i = stack[&x.as_non_ssa()].last().unwrap();
                // s.replace_oper_use_with_ssa(x, *i);
            }
        }

        for a in s.def() {
            if let Oper::Var(_, _) = a {
                let c = count[&a.as_non_ssa()];
                count.insert(a.as_non_ssa(), c + 1);

                let i = c;
                stack.get_mut(&a.as_non_ssa()).unwrap().push(i);
                s.replace_oper_def_with_ssa(a, i);
            }
            // let c = count[&a.as_non_ssa()];
            // count.insert(a.as_non_ssa(), c + 1);

            // let i = c;
            // stack.get_mut(&a.as_non_ssa()).unwrap().push(i);
            // s.replace_oper_def_with_ssa(a, i);
        }
    }

    for y in cfg.graph.succ[&n].iter() {
        let j = cfg.graph.pred[*y].get_index_of(&n).unwrap();
        for s in &cfg.blocks[*y].statements {
            let s = &mut *s.borrow_mut();
            if let Stmt::Tac(_, Expr::Phi(args)) = s {
                let a = args.get_mut(j).unwrap();
                let i = stack[&a.0.as_non_ssa()].last().unwrap();
                a.0.replace_with_ssa(a.0, *i);
                a.1 = n;
            }
        }
    }

    for (x, domi) in cfg.dom_ctx.idom.clone().iter().enumerate() {
        match domi {
            Some(domi) => {
                if *domi == n {
                    rename(cfg, x, count, stack)
                }
            }
            _ => (),
        }
    }

    for s in &cfg.blocks[n].statements {
        for a in s.borrow().def() {
            if let Oper::Var(_, _) = a {
                stack
                    .get_mut(&a.as_non_ssa())
                    .unwrap_or_else(|| panic!("tried unwrapping for: {:?}", a))
                    .pop();
            }
            // stack
            //     .get_mut(&a.as_non_ssa())
            //     .unwrap_or_else(|| panic!("tried unwrapping for: {:?}", a))
            //     .pop();
        }
    }
}

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

// Experimental - Adv ssa destruction to 
fn convert_to_conventional_ssa(cfg: &mut CFG) {
    for bb in &mut cfg.blocks {
        bb.insert_at_beginning(Stmt::ParallelCopy(vec![]));
        bb.insert_at_end(Stmt::ParallelCopy(vec![]));
    }

    for b0_ind in 0..cfg.blocks.len() {
        let b0 = &cfg.blocks[b0_ind];
        // let mut to_remove = vec![];
        for (index, statement) in b0.statements.iter().enumerate() {
            match &mut *statement.borrow_mut() {
                Stmt::Tac(a0, Expr::Phi(args)) => {
                    for (ai, bi) in args.iter_mut() {
                        let pci = cfg.blocks[*bi].statements.last().unwrap();

                        let ai_prime = match ai {
                            Oper::Var(value, _) => Oper::Var(*value, 0),
                            // Oper::Temp(value, _) => Oper::Var(*value, 0),
                            _ => panic!("Expected var in phi function"),
                        };

                        // pci.replace(Stmt::Tac(ai_prime, Expr::Oper(*ai)));
                        match &mut *pci.borrow_mut() {
                            Stmt::ParallelCopy(pcopy) => pcopy
                                .push(Rc::new(RefCell::new(Stmt::Tac(ai_prime, Expr::Oper(*ai))))),
                            _ => panic!(""),
                        }
                        *ai = ai_prime;
                    }
                    // begin
                    let pc0 = b0.statements.first().unwrap();
                    let a0_prime = match a0 {
                        Oper::Var(value, _) => Oper::Var(*value, 0),
                        // Oper::Temp(value, _) => Oper::Var(*value, 0),
                        _ => panic!("Expected var in phi function"),
                    };

                    // pc0.replace(Stmt::Tac(*a0, Expr::Oper(a0_prime)));
                    match &mut *pc0.borrow_mut() {
                        Stmt::ParallelCopy(pcopy) => {
                            pcopy.push(Rc::new(RefCell::new(Stmt::Tac(*a0, Expr::Oper(a0_prime)))))
                        }
                        _ => panic!("expected pc0 to be a parallel copy"),
                    }
                    *a0 = a0_prime;

                    // coalesce
                    // to_remove.push(index);
                }
                _ => (),
            }
        }

        // I hate this but it works for now!
        // cfg.blocks[b0_ind].statements = cfg.blocks[b0_ind]
        //     .statements
        //     .iter()
        //     .enumerate()
        //     .filter(|(i, _)| !to_remove.contains(i))
        //     .map(|(_, v)| v.clone())
        //     .collect();
    }
}

fn remove_phi_functions(cfg: &mut CFG) {
    for b0_ind in 0..cfg.blocks.len() {
        let b0 = &cfg.blocks[b0_ind];
        let mut to_remove = vec![];
        for (index, statement) in b0.statements.clone().iter().enumerate() {
            match &mut *statement.borrow_mut() {
                Stmt::Tac(a0, Expr::Phi(args)) => {
                    for (ai, bi) in args {
                        let stmt = Stmt::Tac(*a0, Expr::Oper(*ai));
                        cfg.blocks[*bi].insert_at_end(stmt);
                    }

                    // remove_phi_func
                    to_remove.push(index);
                }
                _ => (),
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
                    if lval == rval {
                        true
                    } else {
                        false
                    }
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
                let cond = pcopy.iter().find(|stmt2| match &*stmt2.borrow() {
                    Stmt::Tac(c, Expr::Oper(b2)) => {
                        if a != c && b1 == b2 {
                            true
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
                } else {
                    if a != b1 {
                        let aprime = match a {
                            Oper::Var(sym, _) => Oper::Var(*sym, 0),
                            _ => panic!(""),
                        };
                        let copy = Stmt::Tac(aprime, Expr::Oper(*a));
                        seq.push(Rc::new(RefCell::new(copy)));
                        *a = aprime;
                        true
                    } else {
                        false
                    }
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
