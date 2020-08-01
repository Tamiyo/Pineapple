use crate::compiler::control_flow::ControlFlowContext;

use std::collections::HashSet;
use std::iter::FromIterator;

pub fn compute_dominators(context: &mut ControlFlowContext) {
    compute_dom_tree(context);
    compute_idom_tree(context);
    compute_domf_tree(context);
}

fn compute_dom_tree(context: &mut ControlFlowContext) {
    // dominator of the start node is the start of the cfg
    context.dom.insert_dominator(0, 0);
    let len = context.cfg.blocks.len();

    // for all other nodes, set all nodes as the dominators
    for i in 1..len {
        for j in 0..len {
            context.dom.insert_dominator(i, j);
        }
    }

    // iteratively eliminate nodes that are not dominators
    let mut changed = true;
    while changed {
        changed = false;

        for n in 1..len {
            let singleton = HashSet::from_iter(vec![n]);

            let mut diff: HashSet<usize> = HashSet::new();
            for (i, p) in context.cfg.graph.pred.get(&n).unwrap().iter().enumerate() {
                if i == 0 {
                    diff = context.dom.get_dominator(*p).clone();
                } else {
                    diff = diff
                        .intersection(context.dom.get_dominator(*p))
                        .copied()
                        .collect();
                }
            }
            let unioned = diff.union(&singleton).copied().collect();
            let prev = context.dom.get_dominator(n).clone();
            context.dom.set_dominator(n, unioned);
            if context.dom.get_dominator(n) != &prev {
                changed = true;
            }
        }
    }
}

fn compute_idom_tree(context: &mut ControlFlowContext) {
    for n in 1..context.cfg.blocks.len() {
        let mut min_index = 0;
        let mut min_value = context.cfg.blocks.len();
        for e in context.dom.get_dominator(n) {
            if *e != n && n - e < min_value {
                min_value = n - e;
                min_index = *e;
            }
        }
        context.dom.insert_immediate(n, min_index);
    }
}

fn compute_domf_tree(context: &mut ControlFlowContext) {
    let len = context.cfg.blocks.len();
    for b in 0..len {
        if context.cfg.graph.pred.get(&b).unwrap().len() >= 2 {
            for p in context.cfg.graph.pred.get(&b).unwrap().iter() {
                let mut runner = *p;
                while runner != context.dom.get_immediate_at(b, 0) {
                    context.dom.insert_frontier(runner, b);
                    runner = context.dom.get_immediate_at(runner, 0);
                }
            }
        }
    }
}
