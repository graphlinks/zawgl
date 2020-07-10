pub mod traits;
pub mod container;
use std::marker::PhantomData;

use self::traits::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct EdgeIndex {
    index: usize,
}

impl EdgeIndex {
    pub fn new(index: usize) -> Self {
        EdgeIndex {index: index}
    }
}

impl traits::MemGraphId for EdgeIndex {
    fn get_index(&self) -> usize {
        self.index
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct NodeIndex {
    index: usize,
}

impl NodeIndex {
    pub fn new(index: usize) -> Self {
        NodeIndex {index: index}
    }
}

impl traits::MemGraphId for NodeIndex {
    fn get_index(&self) -> usize {
        self.index
    }
}

pub struct VertexData<EID: MemGraphId> {
    pub first_outbound_edge: Option<EID>,
    pub first_inbound_edge: Option<EID>,
}

impl <EID: MemGraphId + Copy> VertexData<EID> {
    pub fn get_first_outbound_edge(&self) -> Option<EID> {
        self.first_outbound_edge
    }
    pub fn get_first_inbound_edge(&self) -> Option<EID> {
        self.first_inbound_edge
    }
}

pub struct EdgeData<NID: MemGraphId, EID: MemGraphId> {
    pub source: NID,
    pub target: NID,
    pub next_outbound_edge: Option<EID>,
    pub next_inbound_edge: Option<EID>,
}

impl <NID: MemGraphId + Copy, EID: MemGraphId + Copy> EdgeData<NID, EID> {
    pub fn get_source(&self) -> NID {
        self.source
    }
    pub fn get_target(&self) -> NID {
        self.target
    }

    pub fn get_next_outbound_edge(&self) -> Option<EID> {
        self.next_outbound_edge
    }
    pub fn get_next_inbound_edge(&self) ->  Option<EID> {
        self.next_inbound_edge
    }
}

pub struct Graph<'g> {
    nodes: Vec<VertexData<EdgeIndex>>,
    edges: Vec<EdgeData<NodeIndex, EdgeIndex>>,
    graph_lifetime: &'g PhantomData<Graph<'g>>,
}

pub struct Successors<'g> {
    graph: &'g Graph<'g>,
    current_edge_index: Option<EdgeIndex>,
}

pub struct Ancestors<'g> {
    graph: &'g Graph<'g>,
    current_edge_index: Option<EdgeIndex>,
}

impl <'graph> Iterator for Successors<'graph> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_index) => {
                let edge = &self.graph.edges[edge_index.get_index()];
                self.current_edge_index = edge.next_outbound_edge;
                Some(edge.target)
            }
        }
    }
}

impl <'graph> Iterator for Ancestors<'graph> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_index) => {
                let edge = &self.graph.edges[edge_index.get_index()];
                self.current_edge_index = edge.next_inbound_edge;
                Some(edge.source)
            }
        }
    }
}

pub struct OutEdges<'g> {
    graph: &'g Graph<'g>,
    current_edge_index: Option<EdgeIndex>,
}

impl <'g> Iterator for OutEdges<'g> {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<EdgeIndex> {
        match self.current_edge_index {
            None => None,
            Some(edge_index) => {
                let edge = &self.graph.edges[edge_index.get_index()];
                let curr_edge_index = self.current_edge_index;
                self.current_edge_index = edge.next_outbound_edge;
                curr_edge_index
            }
        }
    }
}


pub struct InEdges<'g> {
    graph: &'g Graph<'g>,
    current_edge_index: Option<EdgeIndex>,
}

impl <'graph> Iterator for InEdges<'graph> {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_edge_index {
            None => None,
            Some(edge_index) => {
                let edge = &self.graph.edges[edge_index.get_index()];
                let curr_edge_index = self.current_edge_index;
                self.current_edge_index = edge.next_inbound_edge;
                curr_edge_index
            }
        }
    }
}

impl <'g> GraphTrait<'g, NodeIndex, EdgeIndex> for Graph<'g> {
    type OutIt = OutEdges<'g>;
    type InIt = InEdges<'g>;
    fn out_edges(&'g self, source: &NodeIndex) -> Self::OutIt {
        let first_outbound_edge = self.nodes[source.get_index()].first_outbound_edge;
        OutEdges{ graph: self, current_edge_index: first_outbound_edge }
    }

    fn in_edges(&'g self, target: &NodeIndex) -> InEdges {
        let first_inbound_edge = self.nodes[target.get_index()].first_inbound_edge;
        InEdges{ graph: self, current_edge_index: first_inbound_edge }
    }

    fn get_source_index(&self, edge_index: &EdgeIndex) -> &NodeIndex {
        &self.edges[edge_index.get_index()].source
    }
    fn get_target_index(&self, edge_index: &EdgeIndex) -> &NodeIndex {
        &self.edges[edge_index.get_index()].target
    }

    fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    fn edges_len(&self) -> usize {
        self.edges.len()
    }

    fn get_nodes_ids(&self) -> Vec<NodeIndex> {
        (0..self.nodes_len()).map(NodeIndex::new).collect()
    }
    
    fn in_degree(&self, node: &NodeIndex) -> usize {
        self.in_edges(node).count()
    }
    fn out_degree(&self, node: &NodeIndex) -> usize {
        self.out_edges(node).count()
    }
}
impl <'g> Graph<'g> {
    pub fn new() -> Self {
        Graph{ nodes: Vec::new(), edges: Vec::new(), graph_lifetime: &PhantomData }
    }

    pub fn add_vertex(&mut self) -> NodeIndex {
        let index = self.nodes.len();
        self.nodes.push(VertexData::<EdgeIndex>{first_outbound_edge: None, first_inbound_edge: None});
        NodeIndex::new(index)
    }

    pub fn get_vertex(&self, id: NodeIndex) -> &VertexData<EdgeIndex> {
        &self.nodes[id.get_index()]
    }
    pub fn get_edge(&self, id: EdgeIndex) -> &EdgeData<NodeIndex, EdgeIndex> {
        &self.edges[id.get_index()]
    }

    pub fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) -> EdgeIndex {
        let index = self.edges.len();
        {
            let source_data = &self.nodes[source.get_index()];
            let target_data = &self.nodes[target.get_index()];
            self.edges.push(EdgeData{source: source, target: target,
                 next_inbound_edge: target_data.first_inbound_edge, 
                 next_outbound_edge: source_data.first_outbound_edge});
        }
        
        let ms = &mut self.nodes[source.get_index()];
        ms.first_outbound_edge = Some(EdgeIndex::new(index));
        let mt = &mut self.nodes[target.get_index()];
        mt.first_inbound_edge = Some(EdgeIndex::new(index));
        EdgeIndex::new(index)
    }

    pub fn successors(&self, source: &NodeIndex) -> Successors {
        let first_outbound_edge = self.nodes[source.get_index()].first_outbound_edge;
        Successors{ graph: self, current_edge_index: first_outbound_edge }
    }
    
    pub fn ancestors(&self, target: &NodeIndex) -> Ancestors {
        let first_inbound_edge = self.nodes[target.get_index()].first_inbound_edge;
        Ancestors{ graph: self, current_edge_index: first_inbound_edge }
    }
    
    pub fn get_nodes(&self) -> &Vec<VertexData<EdgeIndex>> {
        &self.nodes
    }
    pub fn get_edges(&self) -> &Vec<EdgeData<NodeIndex, EdgeIndex>> {
        &self.edges
    }
}

#[cfg(test)]
mod test_graph {
    use super::*;
    #[test]
    fn test_small_graph_it() {
        let mut graph = Graph::new();
        let n0 = graph.add_vertex();
        let n1 = graph.add_vertex();
        let n2 = graph.add_vertex();

        let e0 = graph.add_edge(n0, n1);
        let e1 = graph.add_edge(n1, n2);
        let e2 = graph.add_edge(n0, n2);

        let ed0 = graph.get_edge(e0);
        assert_eq!(ed0.source, n0);
        assert_eq!(ed0.target, n1);
        assert_eq!(ed0.next_outbound_edge, None);
        
        let nd0 = graph.get_vertex(n0);
        assert_eq!(nd0.first_outbound_edge, Some(e2));

        let ed2 = graph.get_edge(e2);
        assert_eq!(ed2.source, n0);
        assert_eq!(ed2.target, n2);
        assert_eq!(ed2.next_outbound_edge, Some(e0));

        let targets = graph.successors(&n0).collect::<Vec<NodeIndex>>();
        assert_eq!(targets[0], n2);
        assert_eq!(targets[1], n1);
        assert_eq!(targets.len(), 2);

        let sources = graph.ancestors(&n2).collect::<Vec<NodeIndex>>();
        assert_eq!(sources.len(), 2);

    }
}