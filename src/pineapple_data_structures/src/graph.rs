type NodeIndex = usize;

#[derive(Debug, Default, Clone)]
pub struct Graph {
    pub edges: Vec<Vec<NodeIndex>>,
}

impl Graph {
    pub fn add_node(&mut self) -> NodeIndex {
        let index = self.edges.len();
        self.edges.push(vec![]);
        index
    }

    pub fn add_directed_edge(&mut self, a: NodeIndex, b: NodeIndex) {
        assert!(a < self.edges.len() && b < self.edges.len());

        self.edges[a].push(b);
    }

    pub fn successors(&self, n: NodeIndex) -> Vec<NodeIndex> {
        assert!(n < self.edges.len());
        self.edges[n].clone()
    }

    pub fn predecessors(&self, n: NodeIndex) -> Vec<NodeIndex> {
        let predecessors: Vec<NodeIndex> = self
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edges)| if edges.contains(&n) { true } else { false })
            .map(|(node, _)| node)
            .collect();
        predecessors
    }
}
