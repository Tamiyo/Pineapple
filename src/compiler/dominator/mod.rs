use super::flowgraph::cfg::CFG;
use std::collections::HashSet;
use std::fmt;

// Dominators
// https://www.cl.cam.ac.uk/~mr10/lengtarj.pdf
// https://gist.github.com/yuzeh/a5e6602dfdb0db3c2130c10537db54d7
// I don't like dealing with Options here, but idk if there is a proper way of 
// having a "null" value like this. Will look into but this is unimportant.
#[derive(Default)]
pub struct DominatorContext {
    pub dom: Vec<Option<usize>>,
    pub strict_dom: Vec<HashSet<usize>>,

    pub idom: Vec<Option<usize>>,
    pub domf: Vec<HashSet<usize>>,
}

impl DominatorContext {
    pub fn strictly_dominates(&self, a: &usize, b: &usize) -> bool {
        self.strict_dom[*a].contains(b)
    }
}

#[derive(Default)]
struct DominatorState {
    bucket: Vec<HashSet<usize>>,
    dfnum: Vec<usize>,
    vertex: Vec<Option<usize>>,
    parent: Vec<Option<usize>>,
    semi: Vec<Option<usize>>,
    ancestor: Vec<Option<usize>>,
    idom: Vec<Option<usize>>,
    samedom: Vec<Option<usize>>,
    best: Vec<Option<usize>>,
    n: usize,
}
// Implementation of lengauer_tarjan algo
// Adapted from https://gist.github.com/yuzeh/a5e6602dfdb0db3c2130c10537db54d7 and Appel p.414
// This is kinda dirty with Options, I wonder if there is a better way to do this?
pub fn compute_dominator_context(cfg: &mut CFG) {
    let mut ctx = DominatorContext::default();

    compute_dominators(cfg, &mut ctx);
    compute_immediate_dominators(cfg, &mut ctx);
    compute_dominance_frontier(cfg, &mut ctx);

    cfg.dom_ctx = ctx;
}

// Not sure how I feel about inlining the helper functions inside this one...
fn compute_dominators(cfg: &CFG, ctx: &mut DominatorContext) {
    fn dfs(state: &mut DominatorState, p: Option<usize>, n: usize, cfg: &CFG) {
        if state.dfnum[n] == 0 {
            state.dfnum[n] = state.n;
            state.vertex[state.n] = Some(n);
            state.parent[n] = p;
            state.n += 1;

            for w in &cfg.graph.succ[n] {
                dfs(state, Some(n), *w, cfg);
            }
        }
    }

    fn ancestor_with_lowest_semi(state: &mut DominatorState, v: usize) -> Option<usize> {
        if let Some(a) = state.ancestor[v] {
            if let Some(_) = state.ancestor[a] {
                let b = ancestor_with_lowest_semi(state, a).unwrap();
                state.ancestor[v] = state.ancestor[a];

                if state.dfnum[state.semi[b].unwrap()]
                    < state.dfnum[state.semi[state.best[v].unwrap()].unwrap()]
                {
                    state.best[v] = Some(b);
                }
            }
        }
        state.best[v]
    }

    fn link(state: &mut DominatorState, p: Option<usize>, n: usize) {
        state.ancestor[n] = p;
        state.best[n] = Some(n);
    }

    let mut state = DominatorState::default();

    // Init
    state.n = 0;
    let size = cfg.blocks.len();
    state.bucket = vec![HashSet::new(); size];
    state.dfnum = vec![0; size];
    state.vertex = vec![None; size];
    state.parent = vec![None; size];
    state.semi = vec![None; size];
    state.ancestor = vec![None; size];
    state.idom = vec![None; size];
    state.samedom = vec![None; size];
    state.best = vec![None; size];

    // Algorithm
    dfs(&mut state, None, 0, cfg);

    for i in (1..state.n).rev() {
        let n = state.vertex[i].unwrap();
        let p = state.parent[n];
        let mut s = p;

        for v in &cfg.graph.pred[n] {
            let s_prime = if state.dfnum[*v] <= state.dfnum[n] {
                Some(*v)
            } else {
                let lowest = ancestor_with_lowest_semi(&mut state, *v);
                if let Some(lowest) = lowest {
                    state.semi[lowest]
                } else {
                    None
                }
            };

            if let Some(s_prime) = s_prime {
                if state.dfnum[s_prime] < state.dfnum[s.unwrap()] {
                    s = Some(s_prime);
                }
            }
        }

        state.semi[n] = s;
        if let Some(s) = s {
            state.bucket[s].insert(n);
        }

        link(&mut state, p, n);

        if let Some(p) = p {
            for v in state.bucket[p].clone() {
                let y = ancestor_with_lowest_semi(&mut state, v).unwrap();
                if state.semi[y] == state.semi[v] {
                    state.idom[v] = Some(p);
                } else {
                    state.samedom[v] = Some(y);
                }
            }
            state.bucket[p] = HashSet::new();
        }
    }

    for i in 1..(state.n - 1) {
        let n = state.vertex[i].unwrap();
        if let Some(_) = state.samedom[n] {
            state.idom[n] = state.idom[state.samedom[n].unwrap()];
        }
    }

    // This is also pretty inefficient but we'll get it working first
    let mut children: Vec<Vec<usize>> = vec![vec![]; size];
    for (node, parent) in state.parent.iter().enumerate() {
        match parent {
            Some(parent) => {
                children[*parent].push(node);
            }
            _ => (),
        }
    }

    fn strict_traversal(
        target: usize,
        children: &Vec<Vec<usize>>,
        strict: &mut Vec<HashSet<usize>>,
    ) {
        for child in &children[target] {
            strict_traversal(*child, children, strict);
            strict[target].insert(*child);

            let children_of_child = strict[*child].clone();
            strict[target].extend(children_of_child);
        }
    };

    let mut strict: Vec<HashSet<usize>> = vec![HashSet::new(); size];
    strict_traversal(0, &children, &mut strict);

    ctx.strict_dom = strict;
    ctx.dom = state.idom.clone();
}

// I'll be honest I have no idea where I found this algorithm, maybe I made it who knows.
// It does look inefficient though so I'll have to find a proper solution in the future.
fn compute_immediate_dominators(cfg: &CFG, ctx: &mut DominatorContext) {
    let mut idom: Vec<Option<usize>> = vec![None; cfg.blocks.len()];

    for n in 1..cfg.blocks.len() {
        let mut min_index = 0;
        let mut min_value = cfg.blocks.len();
        for e in ctx.dom.get(n).unwrap() {
            if *e != n && n - e < min_value {
                min_value = n - e;
                min_index = *e;
            }
        }
        idom[n] = Some(min_index);
    }

    ctx.idom = idom;
}

// This is pretty efficient according to the SSA book.
fn compute_dominance_frontier(cfg: &CFG, ctx: &mut DominatorContext) {
    let mut domf: Vec<HashSet<usize>> = vec![HashSet::new(); cfg.blocks.len()];

    for (a, connections) in &cfg.graph.edges {
        for b in connections {
            let mut x = Some(*a);

            while x != None && !ctx.strictly_dominates(&x.unwrap(), b) {
                domf[x.unwrap()].insert(*b);
                x = ctx.idom[x.unwrap()];
            }
        }
    }

    ctx.domf = domf;
}

impl fmt::Debug for DominatorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        writeln!(f, "{:?}", self.dom)?;
        writeln!(f, "{:?}", self.idom)?;
        writeln!(f, "{:?}", self.domf)
    }
}
