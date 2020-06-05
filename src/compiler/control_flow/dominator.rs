use std::collections::HashSet;

pub struct Dominator {
    pub tree: Vec<HashSet<usize>>,
    pub immediate: Vec<Vec<usize>>,
    pub frontier: Vec<HashSet<usize>>,
}

impl Dominator {
    pub fn new(size: usize) -> Self {
        Dominator {
            tree: vec![HashSet::new(); size],
            immediate: vec![vec![]; size],
            frontier: vec![HashSet::new(); size],
        }
    }

    pub fn insert_dominator(&mut self, node: usize, v: usize) {
        self.tree[node].insert(v);
    }

    pub fn get_dominator(&self, node: usize) -> &HashSet<usize> {
        &self.tree[node]
    }

    pub fn set_dominator(&mut self, node: usize, v: HashSet<usize>) {
        self.tree[node] = v;
    }

    pub fn insert_immediate(&mut self, node: usize, v: usize) {
        self.immediate[node].push(v);
    }

    pub fn get_immediate_at(&self, node: usize, slot: usize) -> usize {
        self.immediate[node][slot]
    }

    pub fn insert_frontier(&mut self, node: usize, v: usize) {
        self.frontier[node].insert(v);
    }

    pub fn get_frontier(&mut self, node: usize) -> &HashSet<usize> {
        &self.frontier[node]
    }
}
