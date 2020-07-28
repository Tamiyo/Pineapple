use crate::compiler::control_flow::ControlFlowContext;
use crate::compiler::three_address_code::*;
use std::collections::HashSet;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

// Appel p. 418
pub fn dead_code_elimination(context: &mut ControlFlowContext) {
    let mut w: HashSet<Operand> = HashSet::new();
    let mut usages: HashMap<Operand, Vec<(Stmt, usize)>> = HashMap::new();
    let mut definitions: HashMap<Operand, (Stmt, usize)> = HashMap::new();
    for (i, b) in context.cfg.blocks.iter().enumerate() {
        for s in b.statements.iter() {
            for v in &s.vars_defined() {
                w.insert(*v);
                definitions.insert(*v, (s.clone(), i));
            }
            for v in &s.vars_used() {
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

    let mut w: Vec<Operand> = w.iter().copied().collect();

    while !w.is_empty() {
        let v = w.pop().expect("Expected popped value");
        if let Some(t) = usages.get(&v) {
            if t.is_empty() {
                remove_statement_and_patch(v, context, &mut w, &mut definitions, &mut usages)
            }
        } else {
            remove_statement_and_patch(v, context, &mut w, &mut definitions, &mut usages)
        }
    }
}

fn remove_statement_and_patch(
    v: Operand,
    context: &mut ControlFlowContext,
    w: &mut Vec<Operand>,
    definitions: &mut HashMap<Operand, (Stmt, usize)>,
    usages: &mut HashMap<Operand, Vec<(Stmt, usize)>>,
) {
    let (s, b) = match definitions.get(&v) {
        Some(x) => x,
        None => panic!("no def for {:?}", v),
    };
    
    for xi in s.vars_used() {
        if let Entry::Occupied(mut entry) = usages.entry(xi) {
            if let Some(i) = entry.get().iter().position(|r| *r == (s.clone(), *b)) {
                entry.get_mut().remove(i);
            };
        }
        w.push(xi);
    }
    context.cfg.blocks[*b].remove_statement(&s);
}
