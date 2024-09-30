use crate::graph::{AnyNodeId, SyntaxTree};
use crate::visit_side::VisitSide;
use std::collections::VecDeque;
use std::iter::FusedIterator;
use tracing::debug;

enum TraverseState {
    DescendDeeper,
    Exit,
}

pub struct DfsIter<'a, P> {
    inner: DetachedDfsIter,
    tree: &'a SyntaxTree<P>,
}

pub struct DetachedDfsIter {
    stack: VecDeque<(AnyNodeId, TraverseState)>,
    edges_buffer: Vec<AnyNodeId>,
}

impl<'a, P> DfsIter<'a, P> {
    pub(crate) fn new(tree: &'a SyntaxTree<P>, start: AnyNodeId) -> Self {
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
    type Item = (AnyNodeId, VisitSide);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(self.tree)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.stack.len(), Some(self.tree.inner.len() * 2))
    }
}

impl DetachedDfsIter {
    pub fn next<P>(&mut self, tree: &SyntaxTree<P>) -> Option<(AnyNodeId, VisitSide)> {
        let (current_id, descend) = loop {
            let (current_id, descend) = self.stack.pop_back()?;

            if !tree.contains(current_id) {
                debug!(
                    "Node {} has removed from the AST, but still accessible from dfs iterator",
                    current_id
                );
                continue;
            }

            break (current_id, descend);
        };

        if matches!(descend, TraverseState::Exit) {
            return Some((current_id, VisitSide::Exiting));
        }

        self.edges_buffer.clear();
        self.edges_buffer.extend(
            tree.inner
                .children_ids(current_id.into())
                .map(|it| AnyNodeId::from(it)),
        );
        self.edges_buffer.reverse();
        if !self.edges_buffer.is_empty() {
            self.stack.push_back((current_id, TraverseState::Exit));
            for &child in &self.edges_buffer {
                self.stack.push_back((child, TraverseState::DescendDeeper));
            }
            Some((current_id, VisitSide::Entering))
        } else {
            Some((current_id, VisitSide::Leaf))
        }
    }
}

impl<'a, P> FusedIterator for DfsIter<'a, P> {}
