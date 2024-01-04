use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

use fixedbitset::FixedBitSet;
use ghost_cell::GhostToken;
use kodept_core::structure::span::CodeHolder;

use crate::graph::generic_node::{Node, NodeWithParent};
use crate::graph::nodes::{OwnedNode, OwnedNodeImpl, RcNode, RefNode};
use crate::graph::utils::OptVec;
use crate::graph::{GenericASTNode, HasChildrenMarker, Identifiable, NodeId};
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use crate::traits::{Linker, PopulateTree};
use crate::visitor::visit_side::VisitSide;

#[derive(Default)]
pub struct SyntaxTree<'id> {
    id_map: HashMap<NodeId<GenericASTNode>, OwnedNode<'id, GenericASTNode>>,
    root: Option<RcNode<'id, GenericASTNode>>,
    last_id: usize,
}

pub struct ChildScope<'arena, 'id, T> {
    tree: &'arena mut SyntaxTree<'id>,
    node_ref: OwnedNode<'id, GenericASTNode>,
    _phantom: PhantomData<T>,
}

#[derive(Clone)]
pub struct Dfs<'arena, 'id> {
    start: RefNode<'arena, 'id, GenericASTNode>,
    mark: FixedBitSet,
}

pub struct DfsIter<'arena, 'id>(
    Dfs<'arena, 'id>,
    VecDeque<(&'arena GenericASTNode, VisitSide)>,
);

impl<'id> SyntaxTree<'id> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node<T>(&mut self, mut node: T) -> ChildScope<'_, 'id, T>
    where
        T: Into<GenericASTNode> + Identifiable,
    {
        let node_id = NodeId::new(self.last_id);
        node.set_id(node_id);
        let node_ref = OwnedNode::new(node.into(), self.last_id);
        self.id_map.insert(node_id.cast(), node_ref.clone());
        self.last_id += 1;

        ChildScope {
            tree: self,
            node_ref,
            _phantom: PhantomData,
        }
    }

    pub fn children_of<T, U>(&self, id: NodeId<T>) -> OptVec<&U>
    where
        for<'a> &'a U: TryFrom<&'a GenericASTNode>,
    {
        let node = &self.id_map[&id.cast()];
        GhostToken::new(|token| {
            node.borrow(&token)
                .edges
                .iter()
                .filter_map(|x| x.upgrade())
                .map(|x| &x.borrow(&token).data)
                .filter_map(|x| x.try_into().ok())
                .collect()
        })
    }

    pub fn children_of_id<T, U, F, A>(&self, id: NodeId<T>, mut f: F) -> OptVec<A>
    where
        F: FnMut(&mut U) -> A,
        for<'a> &'a mut U: TryFrom<&'a mut GenericASTNode>,
    {
        let mut result = OptVec::Empty;

        GhostToken::new(|mut token| {
            let node_ref = &self.id_map[&id.cast()];
            let mut edges_iter = node_ref
                .borrow(&token)
                .edges
                .iter()
                .filter_map(|x| x.upgrade());
            while let Some(next) = edges_iter.next() {
                let next_data = &mut next.borrow_mut(&mut token).data;
                if let Ok(u) = next_data.try_into() {
                    result.push(f(u));
                }
            }
        });

        result
    }

    pub fn get_mut<T>(&self, id: NodeId<T>) -> Option<&mut T>
    where
        for<'a> &'a mut T: TryFrom<&'a mut GenericASTNode>,
    {
        GhostToken::new(|mut token| {
            let data = &mut self.id_map.get(&id.cast())?.borrow_mut(&mut token).data;
            data.try_into().ok()
        })
    }

    pub fn parent_of<T>(&self, id: NodeId<T>) -> &T::Parent
    where
        T: NodeWithParent + Node,
        for<'a> &'a T::Parent: TryFrom<&'a GenericASTNode>,
    {
        GhostToken::new(|token| {
            self.id_map[&id.cast()]
                .borrow(&token)
                .parent
                .and_then(|x| x.upgrade())
                .map(|x| &x.borrow(&token).data)
                .and_then(|x| x.try_into().ok())
                .unwrap_or_else(|| panic!("Node {:?} has parent with wrong type", id))
        })
    }

    pub fn parent_of_mut<T>(&self, id: NodeId<T>) -> &mut T::Parent
    where
        T: NodeWithParent + Node,
        for<'a> &'a mut T::Parent: TryFrom<&'a mut GenericASTNode>,
    {
        GhostToken::new(|mut token| {
            self.id_map[&id.cast()]
                .borrow(&token)
                .parent
                .and_then(|x| x.upgrade())
                .map(|x| &mut x.borrow_mut(&mut token).data)
                .and_then(|x| x.try_into().ok())
                .unwrap_or_else(|| panic!("Node {:?} has parent with wrong type", id))
        })
    }

    pub fn dfs(&self) -> Dfs {
        Dfs {
            start: self
                .root
                .clone()
                .and_then(|x| x.upgrade())
                .expect("Syntax tree should have at least one node")
                .as_ref(),
            mark: Default::default(),
        }
    }
}

impl<'arena, 'id> Dfs<'arena, 'id> {
    fn with_start(&self, start: RefNode<'arena, 'id, GenericASTNode>) -> Dfs<'arena, 'id> {
        Dfs {
            start,
            mark: self.mark.clone(),
        }
    }

    fn visit(&mut self, id: usize) {
        self.mark.insert(id);
    }

    fn is_visited(&self, id: usize) -> bool {
        self.mark.contains(id)
    }

    pub fn iter(&self) -> DfsIter<'arena, 'id> {
        DfsIter(self.clone(), VecDeque::new())
    }
}

impl<'arena, 'id> Iterator for DfsIter<'arena, 'id> {
    type Item = (&'arena GenericASTNode, VisitSide);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(remaining) = self.1.pop_front() {
                return Some(remaining);
            }
            let OwnedNodeImpl {
                data, uid, edges, ..
            } = GhostToken::new(|token| self.0.start.borrow(&token));

            debug_assert!(!self.0.is_visited(*uid), "Tree cannot contain cycles");
            self.0.visit(*uid);

            match edges {
                OptVec::Empty => self.1.push_back((data, VisitSide::Leaf)),
                OptVec::Single(t) => {
                    self.1.push_back((data, VisitSide::Entering));
                    self.1.extend(
                        t.upgrade()
                            .iter()
                            .flat_map(|t| self.0.with_start(t.as_ref()).iter()),
                    );
                    self.1.push_back((data, VisitSide::Exiting));
                }
                OptVec::Vector(vec) => {
                    self.1.push_back((data, VisitSide::Entering));
                    self.1.extend(
                        vec.into_iter()
                            .filter_map(|x| x.upgrade())
                            .flat_map(|x| self.0.with_start(x.as_ref()).iter()),
                    );
                    self.1.push_back((data, VisitSide::Exiting));
                }
            };
        }
    }
}

impl<'arena, 'id, T> ChildScope<'arena, 'id, T> {
    pub fn add_child<U>(&mut self, mut node: U) -> RcNode<GenericASTNode>
    where
        U: Into<GenericASTNode> + Identifiable,
        T: HasChildrenMarker<U>,
    {
        let node_id = NodeId::new(self.tree.last_id);
        node.set_id(node_id);
        let node_ref =
            OwnedNode::new(node.into(), node_id.into()).with_parent(self.node_ref.share());
        self.tree.id_map.insert(node_id.cast(), node_ref.clone());
        self.tree.last_id += 1;
        self.add_child_by_ref(node_ref.share());
        node_ref.share()
    }

    pub fn add_child_by_ref<U>(&mut self, child_ref: RcNode<GenericASTNode>)
    where
        U: Into<GenericASTNode>,
        T: HasChildrenMarker<U>,
    {
        GhostToken::new(|mut token| {
            let parent = self.node_ref.borrow_mut(&mut token);
            parent.edges.push(child_ref);
        });
    }

    pub fn with_children_from<'b, I, U>(
        mut self,
        iter: I,
        context: &mut (impl Linker<'b> + CodeHolder),
    ) -> Self
    where
        I: IntoIterator<Item = &'b U>,
        U: PopulateTree + 'b,
        T: HasChildrenMarker<<U as PopulateTree>::Output>,
    {
        for item in iter {
            let child_id = item.convert(self.tree, context);
            let child_ref = self.tree.id_map[&child_id.cast()].clone();
            self.add_child_by_ref(child_ref.share());
        }
        self
    }

    pub fn with_rlt<'b, U>(self, context: &mut impl Linker<'b>, rlt_node: U) -> Self
    where
        U: Into<RLTFamily<'b>>,
        NodeId<T>: Into<ASTFamily>,
    {
        context.link_ref(self.id(), rlt_node);
        self
    }

    pub fn id(&self) -> NodeId<T> {
        NodeId::new(GhostToken::new(|token| self.node_ref.borrow(&token).uid))
    }
}
//
// impl Debug for SyntaxTree<'_, '_> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("SyntaxTree")
//             .field("last_id", &self.last_id)
//             .field(
//                 "root",
//                 &GhostToken::new(|token| Some(self.root?.borrow(&token))),
//             )
//             .finish()
//     }
// }
