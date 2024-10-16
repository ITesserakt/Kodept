/// Original code adopted from: https://docs.rs/scene-graph/latest/scene_graph/
mod child_iter;
mod container;
mod detach_iter;
mod petgraph_impls;
mod petgraph_iter;

use crate::dag::child_iter::{DagChildIdIter, DagChildIter};
use crate::dag::detach_iter::DagDetachIter;
use crate::key::CommonKey;
use container::SlotMapContainer;
use slotmap::{Key, SecondaryMap, SlotMap, SparseSecondaryMap};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::RandomState;
use std::mem::replace;
use std::ops::Index;
use thiserror::Error;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum NodeKey {
    Root,
    Child(CommonKey),
}

#[derive(Debug)]
pub struct Node<T, E = ()> {
    pub value: T,
    parent: NodeKey,
    children: Option<Children>,
    last_sibling: Option<CommonKey>,
    next_sibling: Option<CommonKey>,
    pub edge_data: E,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Children {
    first: CommonKey,
    last: CommonKey,
}

#[derive(Debug, PartialEq)]
pub struct DagImpl<N, C> {
    pub root: N,
    arena: C,
    root_children: Option<Children>,
}

pub type Dag<N, E = ()> = DagImpl<N, SlotMap<CommonKey, Node<N, E>>>;
pub type SecondaryDag<N, E = ()> = DagImpl<N, SecondaryMap<CommonKey, Node<N, E>>>;
pub type SparseSecondaryDag<N, E = ()> = DagImpl<N, SparseSecondaryMap<CommonKey, Node<N, E>>>;

pub struct KeyRef<'a, T> {
    value: &'a T,
    id: NodeKey,
}

#[derive(Debug, Error)]
#[error("Parent node not found")]
pub struct ParentNotFound;

impl<T> Clone for KeyRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for KeyRef<'_, T> {}

impl<T, E> DagImpl<T, SlotMap<CommonKey, Node<T, E>>> {
    pub fn add_node_at_root(&mut self, f: impl FnOnce(NodeKey) -> (T, E)) -> NodeKey {
        self.add_node_with_key(NodeKey::Root, f).unwrap()
    }

    pub fn add_node_with_key(
        &mut self,
        parent: NodeKey,
        f: impl FnOnce(NodeKey) -> (T, E),
    ) -> Result<NodeKey, ParentNotFound> {
        let id = self.arena.insert_with_key(|k| {
            let (value, edge) = f(NodeKey::Child(k));
            Node::new(value, parent, edge)
        });
        self.place_node(parent, id)?;
        Ok(NodeKey::Child(id))
    }

    pub fn attach_subgraph_at<C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>>(
        &mut self,
        place_root: NodeKey,
        mut subgraph: DagImpl<T, C>,
        root_edge: E,
    ) -> Result<(NodeKey, HashMap<NodeKey, NodeKey>), ParentNotFound> {
        self.arena.reserve(subgraph.len());
        let other_root = subgraph.root;
        let root_id = self.add_node_with_key(place_root, |_| (other_root, root_edge))?;

        let mut mapping = HashMap::with_capacity(subgraph.arena.len() + 1);
        mapping.insert(NodeKey::Root, root_id);

        for detached_node in
            DagDetachIter::new(&mut subgraph.arena, NodeKey::Root, subgraph.root_children)
        {
            let parent_place = *mapping.get(&detached_node.parent_id).unwrap();
            let new_idx = self
                .add_node_with_key(parent_place, |_| (detached_node.value, detached_node.edge))?;

            mapping.insert(detached_node.id, new_idx);
        }

        Ok((root_id, mapping))
    }

    pub fn consume_map<V, F>(
        mut self,
        mut nodes_map: impl FnMut(T) -> V,
        mut edges_map: impl FnMut(E) -> F,
    ) -> Dag<V, F>
    where
        F: Default,
    {
        let mut new_graph = Dag::new(nodes_map(self.root));
        let mut mapping = HashMap::new();
        mapping.insert(NodeKey::Root, NodeKey::Root);

        for detached_node in DagDetachIter::new(&mut self.arena, NodeKey::Root, self.root_children)
        {
            let parent_id = mapping[&detached_node.parent_id];
            let new_id = new_graph
                .add_node_with_key(parent_id, |_| {
                    (
                        nodes_map(detached_node.value),
                        edges_map(detached_node.edge),
                    )
                })
                .unwrap();
            mapping.insert(detached_node.id, new_id);
        }

        new_graph
    }

    pub fn map<V, F, C>(
        &self,
        mut nodes_map: impl FnMut(NodeKey, &T) -> V,
        mut edges_map: impl FnMut(&E) -> F,
    ) -> DagImpl<V, C>
    where
        C: SlotMapContainer<Key = CommonKey, Data = Node<V, F>>,
        C: FromIterator<(CommonKey, Node<V, F>)>,
    {
        DagImpl {
            root: nodes_map(NodeKey::Root, &self.root),
            arena: self
                .arena
                .iter()
                .map(|(k, v)| {
                    (
                        k,
                        Node {
                            value: nodes_map(NodeKey::Child(k), &v.value),
                            parent: v.parent,
                            children: v.children,
                            last_sibling: v.last_sibling,
                            next_sibling: v.next_sibling,
                            edge_data: edges_map(&v.edge_data),
                        },
                    )
                })
                .collect(),
            root_children: self.root_children,
        }
    }

    pub fn node_weights_mut<const N: usize>(&mut self, ids: [NodeKey; N]) -> Option<[&mut T; N]> {
        assert!(ids.iter().all(|it| !matches!(it, NodeKey::Root)));
        assert_eq!(HashSet::<_, RandomState>::from_iter(&ids).len(), ids.len());

        let keys = ids.map(|it| match it {
            NodeKey::Root => unreachable!(),
            NodeKey::Child(x) => x,
        });
        let refs = self.arena.get_disjoint_mut(keys);

        refs.map(|it| it.map(|node| &mut node.value))
    }
}

impl<T, C> DagImpl<T, C> {
    pub fn len(&self) -> usize
    where
        C: SlotMapContainer,
    {
        self.arena.len() + 1
    }

    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl<T, C, E> DagImpl<T, C>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
{
    pub fn new(root: T) -> Self
    where
        C: Default,
    {
        Self {
            root,
            arena: Default::default(),
            root_children: None,
        }
    }

    pub fn detach_subgraph_at(&mut self, id: NodeKey) -> Option<(Dag<T, E>, E)> {
        let id = match id {
            NodeKey::Root => return None,
            NodeKey::Child(id) => id,
        };

        let node = self.arena.remove(id)?;
        let mut subgraph = DagImpl::new(node.value);
        let mut mapping = HashMap::new();
        mapping.insert(NodeKey::Child(id), NodeKey::Root);

        for detached_node in DagDetachIter::new(&mut self.arena, NodeKey::Child(id), node.children)
        {
            let parent_id = match detached_node.parent_id {
                NodeKey::Root => NodeKey::Root,
                NodeKey::Child(_) => *mapping.get(&detached_node.parent_id)?,
            };
            let new_id = subgraph
                .add_node_with_key(parent_id, |_| (detached_node.value, detached_node.edge))
                .unwrap();
            mapping.insert(detached_node.id, new_id);
        }

        self.fix_parent(node.next_sibling, node.last_sibling, node.parent, id);
        Some((subgraph, node.edge_data))
    }
    
    pub fn replace(&mut self, id: NodeKey, value: T) -> Option<T> {
        match id {
            NodeKey::Root => {
                Some(replace(&mut self.root, value))
            }
            NodeKey::Child(id) => {
                Some(replace(&mut self.arena.get_mut(id)?.value, value))
            }
        }
    }

    /// Removes *all* nodes starting from `id`
    pub fn remove(&mut self, id: NodeKey) {
        let id = match id {
            NodeKey::Root => panic!("Root node cannot be removed"),
            NodeKey::Child(id) => id,
        };

        let Some(node) = self.arena.remove(id) else {
            return;
        };

        for _ in DagDetachIter::new(&mut self.arena, NodeKey::Child(id), node.children) {}

        self.fix_parent(node.next_sibling, node.last_sibling, node.parent, id);
    }

    pub fn contains(&self, id: NodeKey) -> bool {
        match id {
            NodeKey::Root => true,
            NodeKey::Child(id) => self.arena.contains_key(id),
        }
    }

    pub fn node_weight(&self, id: NodeKey) -> Option<&T>
    where
        E: 'static,
    {
        match id {
            NodeKey::Root => Some(&self.root),
            NodeKey::Child(id) => Some(&self.arena.get(id)?.value),
        }
    }

    pub fn node_weight_mut(&mut self, id: NodeKey) -> Option<&mut T>
    where
        E: 'static,
    {
        match id {
            NodeKey::Root => Some(&mut self.root),
            NodeKey::Child(id) => Some(&mut self.arena.get_mut(id)?.value),
        }
    }

    pub fn edge_weight(&self, id: NodeKey) -> Option<&E> {
        match id {
            NodeKey::Root => None,
            NodeKey::Child(id) => Some(&self.arena.get(id)?.edge_data),
        }
    }

    pub fn edge_weight_mut(&mut self, id: NodeKey) -> Option<&mut E> {
        match id {
            NodeKey::Root => None,
            NodeKey::Child(id) => Some(&mut self.arena.get_mut(id)?.edge_data),
        }
    }

    pub fn parent_id(&self, id: NodeKey) -> Option<NodeKey> {
        match id {
            NodeKey::Root => None,
            NodeKey::Child(id) => Some(self.arena.get(id)?.parent),
        }
    }

    pub fn iter_detach_root(&mut self) -> DagDetachIter<T, C, E> {
        DagDetachIter::new(&mut self.arena, NodeKey::Root, self.root_children)
    }

    pub fn iter_detach(&mut self, id: NodeKey) -> DagDetachIter<T, C, E> {
        let children = match id {
            NodeKey::Root => self.root_children.take(),
            NodeKey::Child(id) => self.arena[id].children.take(),
        };
        DagDetachIter::new(&mut self.arena, id, children)
    }

    pub fn children(&self, parent_id: NodeKey) -> DagChildIter<T, C, E> {
        DagChildIter::new(self, parent_id)
    }
    
    pub fn children_ids(&self, parent_id: NodeKey) -> DagChildIdIter<T, C, E> {
        DagChildIdIter::new(self, parent_id)
    }

    fn fix_parent(
        &mut self,
        removed_next_sibling: Option<CommonKey>,
        removed_last_sibling: Option<CommonKey>,
        removed_parent: NodeKey,
        removed_idx: CommonKey,
    ) {
        let parent_children = match removed_parent {
            NodeKey::Root => &mut self.root_children,
            NodeKey::Child(id) => &mut self.arena[id].children,
        };

        if let Some(Children { first, last }) = parent_children {
            // parent has only one child, and [removed_idx] is that child
            if first == last && *first == removed_idx {
                *parent_children = None;
                return;
            }

            if *first == removed_idx {
                *first = removed_next_sibling.unwrap();
            }

            if *last == removed_idx {
                *last = removed_last_sibling.unwrap();
            }
        }

        if let Some(last_sibling) = removed_last_sibling {
            let last_sibling = self.arena.get_mut(last_sibling).unwrap();
            last_sibling.next_sibling = removed_next_sibling;
        }

        if let Some(next_sibling) = removed_next_sibling {
            let next_sibling = self.arena.get_mut(next_sibling).unwrap();
            next_sibling.last_sibling = removed_last_sibling;
        }
    }

    fn place_node(
        &mut self,
        parent: NodeKey,
        node_to_place: CommonKey,
    ) -> Result<(), ParentNotFound> {
        let parent_children = match parent {
            NodeKey::Root => &mut self.root_children,
            NodeKey::Child(id) => &mut self.arena.get_mut(id).ok_or(ParentNotFound)?.children,
        };

        match parent_children {
            None => {
                *parent_children = Some(Children {
                    first: node_to_place,
                    last: node_to_place,
                })
            }
            Some(Children { last, .. }) => {
                let old_last = *last;
                *last = node_to_place;

                let last_sibling = &mut self.arena[old_last];
                last_sibling.next_sibling = Some(node_to_place);

                self.arena[node_to_place].last_sibling = Some(old_last);
            }
        };

        Ok(())
    }
}

impl<T, E> Node<T, E> {
    pub const fn new(value: T, parent: NodeKey, edge: E) -> Self {
        Self {
            value,
            parent,
            children: None,
            last_sibling: None,
            next_sibling: None,
            edge_data: edge,
        }
    }

    pub const fn has_children(&self) -> bool {
        self.children.is_some()
    }

    pub const fn parent_id(&self) -> NodeKey {
        self.parent
    }
}

impl<T, C, E: 'static> Index<NodeKey> for DagImpl<T, C>
where
    C: SlotMapContainer<Key = CommonKey, Data = Node<T, E>>,
{
    type Output = T;

    fn index(&self, index: NodeKey) -> &Self::Output {
        match index {
            NodeKey::Root => &self.root,
            NodeKey::Child(id) => &self.arena[id].value,
        }
    }
}

impl Display for NodeKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKey::Root => write!(f, "root"),
            NodeKey::Child(id) => write!(f, "{:?}", id.data()),
        }
    }
}
