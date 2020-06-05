use crate::compiler::control_flow::ControlFlowGraph;
use crate::compiler::three_address::component::*;
use std::collections::HashSet;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

// Appel p. 418
pub fn dead_code_elimination(cfg: &mut ControlFlowGraph) {
    let mut w: HashSet<SSA> = HashSet::new();
    let mut usages: HashMap<SSA, Vec<(Stmt, usize)>> = HashMap::new();
    let mut definitions: HashMap<SSA, (Stmt, usize)> = HashMap::new();
    for (i, b) in cfg.blocks.iter().enumerate() {
        for s in b.stmts.iter() {
            for v in &s.defined() {
                w.insert(*v);
                definitions.insert(*v, (s.clone(), i));
            }
            for v in &s.used() {
                if !w.contains(&v) {
                    w.insert(*v);
                }
                match usages.entry(*v) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push((s.clone(), i));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![(s.clone(), i)]);
                    }
                };
            }
        }
    }

    let mut w: Vec<SSA> = w.iter().copied().collect();

    while !w.is_empty() {
        let v = w.pop().expect("Expected popped value");
        if let Some(t) = usages.get(&v) {
            if t.is_empty() {
                remove_statement_and_patch(v, cfg, &mut w, &mut definitions, &mut usages)
            }
        } else {
            remove_statement_and_patch(v, cfg, &mut w, &mut definitions, &mut usages)
        }
    }
}

fn remove_statement_and_patch(
    v: SSA,
    cfg: &mut ControlFlowGraph,
    w: &mut Vec<SSA>,
    definitions: &mut HashMap<SSA, (Stmt, usize)>,
    usages: &mut HashMap<SSA, Vec<(Stmt, usize)>>,
) {
    let (s, b) = match definitions.get(&v) {
        Some(x) => x,
        None => panic!("no def for {:?}.{:?} {:?}", v.value, v.ssa, v.is_var),
    };
    for xi in s.used() {
        if let Entry::Occupied(mut entry) = usages.entry(xi) {
            if let Some(i) = entry.get().iter().position(|r| *r == (s.clone(), *b)) {
                entry.get_mut().remove(i);
            };
        }
        w.push(xi);
    }
    cfg.blocks[*b].remove_statement(s.clone());
}
