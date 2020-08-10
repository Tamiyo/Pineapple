use super::interference_graph::InterferenceGraph;
use crate::compiler::{control_flow::ControlFlowContext, three_address_code::Operand};

pub fn register_alloc(ctx: &mut ControlFlowContext, ig: &mut InterferenceGraph) {
    let mut stack: Vec<Operand> = vec![];
    let edges = ig.graph.edges.clone();

    for node in &ig.graph.nodes {
        stack.push(*node);
    }

    ig.graph.clear();
    while !stack.is_empty() {
        let node = stack.pop().unwrap();

        ig.graph.insert(node);

        for dest in &edges[&node] {
            ig.graph.add_edge(node, *dest);
        }

        // Check if colorable
        if let Some(_) = ig.colorable(node) {
            // we good
        } else {
            // Spill
            panic!("Spilling!");
        }
    }
}
