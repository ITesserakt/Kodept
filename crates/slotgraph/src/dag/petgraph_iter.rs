use crate::dag::container::SlotMapContainer;
use crate::dag::{DagImpl, KeyRef, Node, NodeKey};
use crate::key::CommonKey;
use std::iter::FusedIterator;
use std::marker::PhantomData;

pub struct NodeIdentifiersIter<T, I> {
    root_node_id_exhausted: bool,
    inner: I,
    _phantom: PhantomData<T>,
}

impl<'a, T, I> NodeIdentifiersIter<T, I> {
    pub fn new<C>(dag: &'a DagImpl<T, C>) -> Self
    where
        C: SlotMapContainer<Iter<'a> = I>,
    {
        NodeIdentifiersIter {
            root_node_id_exhausted: false,
            inner: dag.arena.iter(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: 'a, I, E: 'a> Iterator for NodeIdentifiersIter<T, I>
where
    I: Iterator<Item = (CommonKey, &'a Node<T, E>)>,
{
    type Item = NodeKey;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.root_node_id_exhausted {
            self.root_node_id_exhausted = true;
            return Some(NodeKey::Root);
        }
        Some(NodeKey::Child(self.inner.next()?.0))
    }
}

impl<'a, T: 'a, I, E: 'a> FusedIterator for NodeIdentifiersIter<T, I> where
    I: FusedIterator<Item = (CommonKey, &'a Node<T, E>)>
{
}

pub struct IntoNodeReferencesIter<'a, T, I> {
    root_node: Option<KeyRef<'a, T>>,
    inner: I,
}

impl<'a, T, I> IntoNodeReferencesIter<'a, T, I> {
    pub fn new<C>(dag: &'a DagImpl<T, C>) -> Self
    where
        C: SlotMapContainer<Iter<'a> = I>,
    {
        Self {
            root_node: Some(KeyRef {
                value: &dag.root,
                id: NodeKey::Root,
            }),
            inner: dag.arena.iter(),
        }
    }
}

impl<'a, T, I, E: 'a> Iterator for IntoNodeReferencesIter<'a, T, I>
where
    I: Iterator<Item = (CommonKey, &'a Node<T, E>)>,
{
    type Item = KeyRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.root_node.take() {
            return Some(root);
        }
        let node = self.inner.next()?;
        Some(KeyRef {
            value: &node.1.value,
            id: NodeKey::Child(node.0),
        })
    }
}

impl<'a, T, I, E: 'a> FusedIterator for IntoNodeReferencesIter<'a, T, I> where
    I: FusedIterator<Item = (CommonKey, &'a Node<T, E>)>
{
}

pub struct IntoEdgeReferencesIter<'a, T, I, E>
where
    T: 'a,
    I: Iterator<Item = (CommonKey, &'a Node<T, E>)>,
    E: 'a,
{
    iter: I,
}

impl<'a, T, I, E> Iterator for IntoEdgeReferencesIter<'a, T, I, E>
where
    T: 'a,
    I: Iterator<Item = (CommonKey, &'a Node<T, E>)>,
{
    type Item = (NodeKey, NodeKey, &'a E);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.iter.next()?;
        let (a, b) = (node.1.parent, NodeKey::Child(node.0));
        Some((a, b, &node.1.edge_data))
    }
}

impl<'a, T, I, E> IntoEdgeReferencesIter<'a, T, I, E>
where
    T: 'a,
    I: Iterator<Item = (CommonKey, &'a Node<T, E>)>,
{
    pub fn new<C>(dag: &'a DagImpl<T, C>) -> Self
    where
        C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>, Iter<'a> = I> + 'a,
    {
        Self {
            iter: dag.arena.iter(),
        }
    }
}

impl<'a, T, I, E> FusedIterator for IntoEdgeReferencesIter<'a, T, I, E>
where
    T: 'a,
    I: FusedIterator<Item = (CommonKey, &'a Node<T, E>)>,
{
}
