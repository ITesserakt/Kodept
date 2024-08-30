use crate::dag::container::SlotMapContainer;
use crate::dag::petgraph_iter::{
    IntoEdgeReferencesIter, IntoNodeReferencesIter, NodeIdentifiersIter,
};
use crate::dag::{DagImpl, KeyRef, Node, NodeKey};
use crate::key::CommonKey;
use petgraph::visit::{
    Data, GraphBase, GraphProp, IntoEdgeReferences, IntoNodeIdentifiers, IntoNodeReferences,
    NodeCount, NodeIndexable, NodeRef,
};
use petgraph::Directed;

impl<T, C: SlotMapContainer> GraphBase for DagImpl<T, C> {
    type EdgeId = (NodeKey, NodeKey);
    type NodeId = NodeKey;
}

impl<T, C: SlotMapContainer> GraphProp for DagImpl<T, C> {
    type EdgeType = Directed;
}

impl<T, C: SlotMapContainer> NodeCount for DagImpl<T, C> {
    fn node_count(&self) -> usize {
        self.len()
    }
}

impl<T, E, C: SlotMapContainer<Data = Node<T, E>>> Data for DagImpl<T, C> {
    type NodeWeight = T;
    type EdgeWeight = E;
}

impl<'a, T, E: 'a, C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>> IntoNodeIdentifiers
    for &'a DagImpl<T, C>
{
    type NodeIdentifiers = NodeIdentifiersIter<T, C::Iter<'a>>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiersIter::new(self)
    }
}

impl<'a, T> NodeRef for KeyRef<'a, T> {
    type NodeId = NodeKey;
    type Weight = T;

    fn id(&self) -> Self::NodeId {
        self.id
    }

    fn weight(&self) -> &Self::Weight {
        self.value
    }
}

impl<'a, T, E: 'a, C> IntoNodeReferences for &'a DagImpl<T, C>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
{
    type NodeRef = KeyRef<'a, T>;
    type NodeReferences = IntoNodeReferencesIter<'a, T, C::Iter<'a>>;

    fn node_references(self) -> Self::NodeReferences {
        IntoNodeReferencesIter::new(self)
    }
}

impl<'a, T, C, E: 'a> IntoEdgeReferences for &'a DagImpl<T, C>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
{
    type EdgeRef = (NodeKey, NodeKey, &'a E);
    type EdgeReferences = IntoEdgeReferencesIter<'a, T, C::Iter<'a>, E>;

    fn edge_references(self) -> Self::EdgeReferences {
        IntoEdgeReferencesIter::new(self)
    }
}

#[cfg(target_pointer_width = "64")]
impl<T, C> NodeIndexable for DagImpl<T, C>
where
    C: SlotMapContainer,
{
    fn node_bound(&self) -> usize {
        self.len()
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        match a {
            // value = 0xFFFF_FFFF
            // version = 0x3
            NodeKey::Root => 0x0000_0003_FFFF_FFFF,
            NodeKey::Child(id) => id.to_index() as usize,
        }
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        match i as u64 {
            // value = 0xFFFF_FFFF
            // version = 0x3
            0x0000_0003_FFFF_FFFF  => NodeKey::Root,
            i => NodeKey::Child(CommonKey::from_index(i)),
        }
    }
}
