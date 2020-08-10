use crate::compiler::three_address_code::Operand;
use crate::graph::{DirectedGraph, UndirectedGraph};
use crate::vm::NUM_REGISTERS;
use std::collections::HashMap;
use std::{collections::HashSet, fmt};

pub struct InterferenceGraph {
    pub graph: UndirectedGraph<Operand>,
    pub colors: HashMap<Operand, usize>,
}

impl InterferenceGraph {
    pub fn new() -> Self {
        InterferenceGraph {
            graph: UndirectedGraph::new(),
            colors: HashMap::new(),
        }
    }

    pub fn colorable(&mut self, operand: Operand) -> Option<usize> {
        let mut registers: Vec<Option<usize>> = (0..NUM_REGISTERS).map(|x| Some(x)).collect();

        for n in &self.graph.edges[&operand] {
            let reg_of_neighbor = self.colors[n];
            registers[reg_of_neighbor] = None;
        }

        for reg in &registers {
            if let Some(r) = reg {
                self.colors.insert(operand, *r);
                return Some(*r);
            }
        }
        None
    }
}

impl fmt::Debug for InterferenceGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.graph)
    }
}
