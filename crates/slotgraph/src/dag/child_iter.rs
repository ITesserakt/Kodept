use crate::dag::container::SlotMapContainer;
use crate::dag::{DagImpl, Node, NodeKey};
use crate::key::CommonKey;
use std::iter::FusedIterator;

pub struct DagChildIter<'a, T, C, E = ()>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
{
    inner: &'a DagImpl<T, C>,
    current_node: Option<CommonKey>,
}

impl<'a, T, C, E> DagChildIter<'a, T, C, E>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
{
    pub fn new(dag: &'a DagImpl<T, C>, parent_id: NodeKey) -> Self {
        let children = match parent_id {
            NodeKey::Root => dag.root_children.as_ref(),
            NodeKey::Child(id) => dag.arena[id].children.as_ref(),
        };
        Self {
            inner: dag,
            current_node: children.map(|it| it.first),
        }
    }
}

impl<'a, T, C, E> Iterator for DagChildIter<'a, T, C, E>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
    E: 'a
{
    type Item = (NodeKey, &'a Node<T, E>);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.current_node?;
        let node = &self.inner.arena[id];
        self.current_node = node.next_sibling;

        Some((NodeKey::Child(id), node))
    }
}

impl<'a, T, C, E> FusedIterator for DagChildIter<'a, T, C, E> where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
    E: 'a
{
}