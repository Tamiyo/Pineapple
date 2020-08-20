use indexmap::{IndexMap, IndexSet};
use std::hash::Hash;

pub trait Graph<T: Clone + Hash + Eq> {
    fn insert(&mut self, a: T);
    fn remove(&mut self, a: &T);
    fn create_edge(&mut self, a: T, b: T);
    fn remove_edge(&mut self, a: &T, b: &T);
    fn incoming_edges_of(&self, a: &T) -> Vec<(T, T)>;

    // Useful for SSA Construction / Destruction
    // Insert c in between (a, b)
    fn insert_in_between(&mut self, a: &T, b: &T, c: &T);
}

pub struct DirectedGraph<T: Clone + Hash + Eq> {
    pub pred: IndexMap<T, IndexSet<T>>,
    pub succ: IndexMap<T, IndexSet<T>>,
    pub nodes: IndexSet<T>,
    pub edges: IndexMap<T, IndexSet<T>>,
}

impl<T: Copy + Hash + Eq> DirectedGraph<T> {
    pub fn new() -> Self {
        DirectedGraph {
            pred: IndexMap::new(),
            succ: IndexMap::new(),
            nodes: IndexSet::new(),
            edges: IndexMap::new(),
        }
    }
}

impl<T: Copy + Hash + Eq> Graph<T> for DirectedGraph<T> {
    fn insert(&mut self, a: T) {
        self.nodes.insert(a);
        self.pred.insert(a, IndexSet::new());
        self.succ.insert(a, IndexSet::new());
        self.edges.insert(a, IndexSet::new());
    }

    fn remove(&mut self, a: &T) {
        self.nodes.remove(a);
        self.pred.get_mut(a).unwrap().clear();
        self.succ.get_mut(a).unwrap().clear();
        self.edges.remove(a);
        for (_, v) in self.edges.iter_mut() {
            v.remove(a);
        }
    }

    fn create_edge(&mut self, a: T, b: T) {
        if self.nodes.contains(&a) && self.nodes.contains(&b) {
            match self.edges.get_mut(&a) {
                Some(v) => v.insert(b),
                _ => false,
            };

            match self.succ.get_mut(&a) {
                Some(v) => v.insert(b),
                _ => false,
            };

            match self.pred.get_mut(&b) {
                Some(v) => v.insert(a),
                _ => false,
            };
        }
    }

    fn remove_edge(&mut self, a: &T, b: &T) {
        self.edges.get_mut(a).unwrap().remove(b);
        self.pred[b].remove(a);
        self.succ[a].remove(b);
    }

    fn incoming_edges_of(&self, a: &T) -> Vec<(T, T)> {
        let mut inc_edges: Vec<(T, T)> = Vec::new();

        for (source, destinations) in &self.edges {
            if destinations.contains(a) {
                inc_edges.push((*source, *a));
            }
        }

        inc_edges
    }

    fn insert_in_between(&mut self, a: &T, b: &T, c: &T) {
        self.remove_edge(a, b);
        self.create_edge(*a, *c);
        self.create_edge(*c, *b);
    }
}
