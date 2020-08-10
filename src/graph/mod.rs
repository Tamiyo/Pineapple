use std::cmp::Eq;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use std::fmt::Debug;

pub struct DirectedGraph<T: Debug + Copy + Clone + Hash + Eq> {
    pub pred: HashMap<T, HashSet<T>>,
    pub succ: HashMap<T, HashSet<T>>,
    pub nodes: HashSet<T>,
    pub edges: HashMap<T, HashSet<T>>,
}

impl<T: Debug + Copy + Clone + Hash + Eq> DirectedGraph<T> {
    pub fn new() -> Self {
        DirectedGraph {
            pred: HashMap::new(),
            succ: HashMap::new(),
            nodes: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    pub fn insert(&mut self, a: T) {
        self.nodes.insert(a.clone());
        self.pred.insert(a, HashSet::new());
        self.succ.insert(a, HashSet::new());
        self.edges.insert(a, HashSet::new());
    }

    pub fn remove(&mut self, a: T) {
        self.nodes.remove(&a);
        self.pred.get_mut(&a).unwrap().clear();
        self.succ.get_mut(&a).unwrap().clear();
        self.edges.remove(&a);
        for (_, v) in self.edges.iter_mut() {
            v.remove(&a);
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.pred.clear();
        self.succ.clear();
        self.edges.clear();
    }

    pub fn add_edge(&mut self, a: T, b: T) {
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
}

impl<T: Debug + Copy + Clone + Hash + Eq> Debug for DirectedGraph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "nodes: {:?}\npred: {:?}\nsucc: {:?}\nedges: {:?}\n",
            self.nodes, self.pred, self.succ, self.edges
        )
    }
}

pub struct UndirectedGraph<T: Debug + Copy + Clone + Hash + Eq> {
    pub nodes: Vec<T>,
    pub edges: HashMap<T, HashSet<T>>,
}

impl<T: Debug + Copy + Clone + Hash + Eq> UndirectedGraph<T> {
    pub fn new() -> Self {
        UndirectedGraph {
            nodes: Vec::new(),
            edges: HashMap::new(),
        }
    }

    pub fn insert(&mut self, a: T) {
        if !self.nodes.contains(&a) {
            self.nodes.push(a.clone());
            self.edges.insert(a, HashSet::new());
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    pub fn add_edge(&mut self, a: T, b: T) {
        match self.edges.get_mut(&a) {
            Some(v) => v.insert(b),
            _ => false,
        };
        match self.edges.get_mut(&b) {
            Some(v) => v.insert(a),
            _ => false,
        };
    }
}

impl<T: Debug + Copy + Clone + Hash + Eq> Debug for UndirectedGraph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "nodes: {:?}\nedges: {:?}\n", self.nodes, self.edges)
    }
}
