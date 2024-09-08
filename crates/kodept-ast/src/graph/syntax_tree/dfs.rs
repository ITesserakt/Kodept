use crate::graph::{GenericNodeId, SyntaxTree};
use crate::visit_side::VisitSide;
use slotgraph::dag::NodeKey;
use std::collections::VecDeque;
use std::iter::FusedIterator;

pub enum TraverseState {
    DescendDeeper,
    Exit,
}

pub struct DfsIter<'a, P> {
    inner: DetachedDfsIter,
    tree: &'a SyntaxTree<P>,
}

pub struct DetachedDfsIter {
    stack: VecDeque<(NodeKey, TraverseState)>,
    edges_buffer: Vec<NodeKey>,
}

impl<'a, P> DfsIter<'a, P> {
    pub fn new(tree: &'a SyntaxTree<P>, start: NodeKey) -> Self {
        let mut stack = VecDeque::with_capacity(tree.inner.len());
        stack.push_back((start, TraverseState::DescendDeeper));

        Self {
            inner: DetachedDfsIter {
                stack,
                edges_buffer: vec![],
            },
            tree,
        }
    }
    
    pub fn detach(self) -> DetachedDfsIter {
        self.inner
    }
}

impl<'a, P> Iterator for DfsIter<'a, P> {
    type Item = (GenericNodeId, VisitSide);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(self.tree)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.stack.len(), Some(self.tree.inner.len() * 2))
    }
}

impl DetachedDfsIter {
    pub fn next<P>(&mut self, tree: &SyntaxTree<P>) -> Option<(GenericNodeId, VisitSide)> {
        let (next, descend) = self.stack.pop_back()?;
        let current_id = next.into();
        if matches!(descend, TraverseState::Exit) {
            return Some((current_id, VisitSide::Exiting));
        }

        self.edges_buffer.clear();
        self.edges_buffer
            .extend(tree.inner.children(next).map(|it| it.0));
        self.edges_buffer.reverse();
        let edges_iter = self.edges_buffer.iter();
        if edges_iter.len() != 0 {
            self.stack.push_back((next, TraverseState::Exit));
            for &child in edges_iter {
                self.stack.push_back((child, TraverseState::DescendDeeper));
            }
            Some((current_id, VisitSide::Entering))
        } else {
            Some((current_id, VisitSide::Leaf))
        }
    }
}

impl<'a, P> FusedIterator for DfsIter<'a, P> {}