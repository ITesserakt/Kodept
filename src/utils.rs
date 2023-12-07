pub mod graph {
    use petgraph::visit::{
        FilterNode, IntoEdgesDirected, IntoNodeIdentifiers, NodeFiltered, VisitMap, Visitable,
    };
    use petgraph::Direction;

    fn leaves<G, NodeId>(graph: G) -> Vec<NodeId>
    where
        G: IntoNodeIdentifiers<NodeId = NodeId>,
        G: IntoEdgesDirected<NodeId = NodeId>,
        NodeId: Clone,
    {
        graph
            .node_identifiers()
            .filter(|it| {
                graph
                    .edges_directed(it.clone(), Direction::Incoming)
                    .count()
                    == 0
            })
            .collect()
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
            let leaves = leaves(&filter);
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

pub mod progress {
    use indicatif::ProgressBar;

    pub trait ProgressEmitter {
        fn update<F>(&self, f: F)
        where
            F: Fn(&ProgressBar);
    }
}
