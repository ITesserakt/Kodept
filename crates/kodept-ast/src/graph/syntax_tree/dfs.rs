use crate::graph::nodes::NodeCell;
use crate::graph::syntax_tree::Graph;
use crate::visit_side::VisitSide;
use slotgraph::dag::NodeKey;
use std::collections::VecDeque;
use std::iter::FusedIterator;

pub enum TraverseState {
    DescendDeeper,
    Exit,
}

pub struct DfsIter<'a> {
    stack: VecDeque<(NodeKey, TraverseState)>,
    edges_buffer: Vec<NodeKey>,
    graph: &'a Graph,
}

impl<'a> DfsIter<'a> {
    pub fn new(graph: &'a Graph, start: NodeKey) -> Self {
        let mut stack = VecDeque::with_capacity(graph.len());
        stack.push_back((start, TraverseState::DescendDeeper));

        Self {
            stack,
            edges_buffer: vec![],
            graph,
        }
    }
}

impl<'a> Iterator for DfsIter<'a> {
    type Item = (&'a NodeCell, VisitSide);

    fn next(&mut self) -> Option<Self::Item> {
        let (next, descend) = self.stack.pop_back()?;
        let current = self.graph.node_weight(next)?;
        if matches!(descend, TraverseState::Exit) {
            return Some((current, VisitSide::Exiting));
        }

        self.edges_buffer.clear();
        self.edges_buffer
            .extend(self.graph.children(next).map(|it| it.0));
        self.edges_buffer.reverse();
        let edges_iter = self.edges_buffer.iter();
        if edges_iter.len() != 0 {
            self.stack.push_back((next, TraverseState::Exit));
            for child in edges_iter {
                self.stack.push_back((*child, TraverseState::DescendDeeper));
            }
            Some((current, VisitSide::Entering))
        } else {
            Some((current, VisitSide::Leaf))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.stack.len(), Some(self.graph.len() * 2))
    }
}

impl<'a> FusedIterator for DfsIter<'a> {}