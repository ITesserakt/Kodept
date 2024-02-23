pub mod graph {
    use itertools::Itertools;
    use petgraph::Direction;
    use petgraph::visit::{
        FilterNode, IntoEdgesDirected, IntoNodeIdentifiers, NodeFiltered, Visitable, VisitMap,
    };

    pub fn roots<'a, G: 'a, NodeId>(graph: G) -> impl Iterator<Item = NodeId> + 'a
    where
        G: IntoNodeIdentifiers<NodeId = NodeId>,
        G: IntoEdgesDirected<NodeId = NodeId>,
        NodeId: Clone,
    {
        graph.node_identifiers().filter(move |it| {
            graph
                .edges_directed(it.clone(), Direction::Incoming)
                .count()
                == 0
        })
    }

    pub fn topological_layers<G>(graph: G) -> Vec<Vec<G::NodeId>>
    where
        G: Visitable,
        G::Map: FilterNode<G::NodeId>,
        G: IntoNodeIdentifiers,
        G: IntoEdgesDirected,
    {
        struct InvertedVisitMap<'m, M>(&'m M);

        impl<'m, N, M> FilterNode<N> for InvertedVisitMap<'m, M>
        where
            M: FilterNode<N>,
        {
            fn include_node(&self, node: N) -> bool {
                !self.0.include_node(node)
            }
        }

        let mut map = graph.visit_map();
        let mut layers = vec![];
        loop {
            let filter = NodeFiltered(graph, InvertedVisitMap(&map));
            let leaves = roots(&filter).collect_vec();
            if leaves.is_empty() {
                break;
            }
            leaves.iter().for_each(|&n| {
                map.visit(n);
            });
            layers.push(leaves);
        }
        layers
    }
}
