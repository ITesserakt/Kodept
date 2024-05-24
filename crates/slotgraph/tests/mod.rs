use petgraph::visit::{EdgeCount, NodeCount};
use petgraph::Direction;
use rstest::rstest;
use slotgraph::{DiGraph, UnGraph};

#[allow(non_snake_case)]
#[rstest]
fn test_undirected() {
    let mut graph = UnGraph::new();

    let a = graph.add_node(1);
    let b = graph.add_node(2);
    let c = graph.add_node(3);

    let A = graph.add_edge(a, b, ());
    let B = graph.add_edge(a, c, ());
    let C = graph.add_edge(b, c, ());

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 3);

    assert_eq!(
        graph.externals(Direction::Incoming).collect::<Vec<_>>(),
        vec![a]
    );
    assert_eq!(
        graph.externals(Direction::Outgoing).collect::<Vec<_>>(),
        vec![c]
    );

    assert_eq!(graph.children(a).collect::<Vec<_>>(), vec![(A, b), (B, c)]);
    assert_eq!(graph.children(b).collect::<Vec<_>>(), vec![(A, a), (C, c)]);
    assert_eq!(graph.children(c).collect::<Vec<_>>(), vec![(B, a), (C, b)]);
}

#[rstest]
fn test_directed() {
    let mut graph = DiGraph::new();

    let a = graph.add_node(1);
    let b = graph.add_node(2);
    let c = graph.add_node(3);

    let A = graph.add_edge(a, b, ());
    let B = graph.add_edge(a, c, ());
    let C = graph.add_edge(b, c, ());

    assert_eq!(graph.children(a).collect::<Vec<_>>(), vec![(A, b), (B, c)]);
    assert_eq!(graph.children(b).collect::<Vec<_>>(), vec![(C, c)]);
    assert_eq!(graph.children(c).collect::<Vec<_>>(), vec![]);
}
