use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

use fixedbitset::FixedBitSet;
use tracing::warn;

use kodept_core::structure::span::CodeHolder;

use crate::graph::generic_node::{Node, NodeWithParent};
use crate::graph::nodes::{BorrowedNode, GhostToken, OwnedNode, RefNode};
use crate::graph::utils::OptVec;
use crate::graph::{GenericASTNode, HasChildrenMarker, Identifiable, NodeId};
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use crate::traits::{Linker, PopulateTree};
use crate::visitor::visit_side::VisitSide;

#[derive(Debug)]
pub struct BuildingStage(GhostToken);
#[derive(Default, Debug)]
pub struct AccessingStage;

#[derive(Debug)]
pub struct SyntaxTree<Stage = AccessingStage> {
    id_map: HashMap<NodeId<GenericASTNode>, OwnedNode>,
    root: Option<BorrowedNode<GenericASTNode>>,
    last_id: usize,
    stage: Stage,
}

pub type SyntaxTreeBuilder = SyntaxTree<BuildingStage>;

pub struct ChildScope<'arena, T> {
    tree: &'arena mut SyntaxTree<BuildingStage>,
    node_ref: OwnedNode,
    _phantom: PhantomData<T>,
}

#[derive(Clone)]
pub struct Dfs<'arena> {
    start: BorrowedNode,
    mark: FixedBitSet,
    _phantom: PhantomData<&'arena ()>,
}

impl Default for SyntaxTree<BuildingStage> {
    fn default() -> Self {
        Self {
            id_map: Default::default(),
            root: None,
            last_id: 0,
            stage: BuildingStage(GhostToken::new()),
        }
    }
}

impl SyntaxTree<BuildingStage> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node<T>(&mut self, mut node: T) -> ChildScope<'_, T>
    where
        T: Into<GenericASTNode> + Identifiable,
    {
        let node_id = NodeId::new(self.last_id);
        node.set_id(node_id);
        let node_ref = OwnedNode::new(node.into(), self.last_id);
        self.id_map.insert(node_id.cast(), node_ref.clone());
        self.last_id += 1;

        if self.root.is_none() {
            self.root = Some(node_ref.share())
        }

        ChildScope {
            tree: self,
            node_ref,
            _phantom: PhantomData,
        }
    }

    pub fn build(self) -> SyntaxTree<AccessingStage> {
        SyntaxTree {
            id_map: self.id_map,
            root: self.root,
            last_id: self.last_id,
            stage: AccessingStage,
        }
    }
}

impl SyntaxTree {
    pub fn dfs(&self) -> Dfs {
        Dfs {
            start: self
                .root
                .clone()
                .expect("Syntax tree should have at least one node"),
            mark: FixedBitSet::with_capacity(self.last_id),
            _phantom: PhantomData,
        }
    }

    pub fn children_of<'b, T, U>(&'b self, id: NodeId<T>, token: &'b GhostToken) -> OptVec<&U>
    where
        for<'a> &'a U: TryFrom<&'a GenericASTNode>,
    {
        let node = &self.id_map[&id.cast()];
        let edges = &node.ro(token).edges;
        edges
            .iter()
            .filter_map(|x| x.upgrade())
            .map(|x| x.ro(token).uid)
            .map(|x| &self.id_map[&NodeId::new(x)].ro(token).data)
            .filter_map(|x| x.try_into().ok())
            .collect()
    }

    pub fn children_of_id<T, U, F, A>(&self, id: NodeId<T>, mut f: F) -> OptVec<A>
    where
        F: FnMut(&mut U) -> A,
        for<'a> &'a mut U: TryFrom<&'a mut GenericASTNode>,
    {
        todo!()
        // let mut result = OptVec::Empty;
        //
        // GhostToken::new(|mut token| {
        //     let node_ref = &self.id_map[&id.cast()];
        //     let mut edges_iter = node_ref
        //         .borrow(&token)
        //         .edges
        //         .iter()
        //         .filter_map(|x| x.upgrade());
        //     while let Some(next) = edges_iter.next() {
        //         let next_data = &mut next.borrow_mut(&mut token).data;
        //         if let Ok(u) = next_data.try_into() {
        //             result.push(f(u));
        //         }
        //     }
        // });
        //
        // result
    }

    pub fn get_mut<'b, T>(&'b self, id: NodeId<T>, token: &'b mut GhostToken) -> Option<&mut T>
    where
        for<'a> &'a mut T: TryFrom<&'a mut GenericASTNode>,
    {
        let node = self.id_map.get(&id.cast())?;
        let data = &mut node.rw(token).data;
        data.try_into().ok()
    }

    pub fn parent_of<'a, T>(&'a self, id: NodeId<T>, token: &'a GhostToken) -> &T::Parent
    where
        T: NodeWithParent + Node,
        for<'b> &'b T::Parent: TryFrom<&'b GenericASTNode>,
    {
        let node = &self.id_map[&id.cast()];
        let parent = &node.ro(token).parent;
        let parent = &parent.as_ref().and_then(|x| x.upgrade()).expect(
            "NodeWithParent contract violated: such kind of nodes should always have a parent",
        );
        let parent_id = &parent.ro(token).uid;
        let parent = &self.id_map[&NodeId::new(*parent_id)];
        let parent = parent.data(token);
        parent.try_into().ok().expect("Node has wrong type")
    }

    pub fn parent_of_mut<T>(&self, id: NodeId<T>) -> &mut T::Parent
    where
        T: NodeWithParent + Node,
        for<'a> &'a mut T::Parent: TryFrom<&'a mut GenericASTNode>,
    {
        todo!()
        // GhostToken::new(|mut token| {
        //     self.id_map[&id.cast()]
        //         .borrow(&token)
        //         .parent
        //         .and_then(|x| x.upgrade())
        //         .map(|x| &mut x.borrow_mut(&mut token).data)
        //         .and_then(|x| x.try_into().ok())
        //         .unwrap_or_else(|| panic!("Node {:?} has parent with wrong type", id))
        // })
    }
}

impl<'arena> Dfs<'arena> {
    fn visit(&mut self, id: usize) {
        self.mark.insert(id);
    }

    fn is_visited(&self, id: usize) -> bool {
        self.mark.contains(id)
    }

    pub fn iter(&mut self, token: &GhostToken, mut f: impl FnMut(&GenericASTNode, VisitSide)) {
        self.dfs_impl(&self.start.clone(), token, &mut |node, side, token| {
            f(&node.ro(token).data, side)
        });
    }

    pub fn iter_mut(
        &mut self,
        token: &mut GhostToken,
        mut f: impl FnMut(&mut GenericASTNode, VisitSide),
    ) {
        self.dfs_mut_impl(self.start.clone(), token, &mut |node, side, token| {
            f(&mut node.rw(token).data, side);
        });
    }

    fn dfs_mut_impl(
        &mut self,
        current: BorrowedNode,
        token: &mut GhostToken,
        mut f: &mut dyn FnMut(RefNode<GenericASTNode>, VisitSide, &mut GhostToken),
    ) {
        let Some(current) = current.upgrade() else {
            return;
        };
        let id = current.ro(token).uid;
        debug_assert!(!self.is_visited(id), "Syntax tree cannot contain cycles");
        self.visit(id);

        match &current.ro(token).edges {
            OptVec::Empty => f(&current, VisitSide::Leaf, token),
            OptVec::Single(child_weak) => {
                let child = child_weak.clone();
                f(&current, VisitSide::Entering, token);
                self.dfs_mut_impl(child, token, &mut f);
                f(&current, VisitSide::Exiting, token);
            }
            OptVec::Vector(vec) => {
                let vec = vec.clone();
                f(&current, VisitSide::Entering, token);
                for child_weak in vec {
                    self.dfs_mut_impl(child_weak.clone(), token, &mut f);
                }
                f(&current, VisitSide::Exiting, token);
            }
        };
    }

    fn dfs_impl(
        &mut self,
        current: &BorrowedNode,
        token: &GhostToken,
        mut f: &mut dyn FnMut(RefNode<GenericASTNode>, VisitSide, &GhostToken),
    ) {
        let Some(current) = current.upgrade() else {
            warn!("Node commit suicide");
            return;
        };

        let id = current.ro(token).uid;
        debug_assert!(!self.is_visited(id), "Syntax tree cannot contain cycles");
        self.visit(id);

        match &current.ro(token).edges {
            OptVec::Empty => f(&current, VisitSide::Leaf, token),
            OptVec::Single(child_weak) => {
                f(&current, VisitSide::Entering, token);
                self.dfs_impl(child_weak, token, &mut f);
                f(&current, VisitSide::Exiting, token);
            }
            OptVec::Vector(vec) => {
                f(&current, VisitSide::Entering, token);
                for child_weak in vec {
                    self.dfs_impl(child_weak, token, &mut f);
                }
                f(&current, VisitSide::Exiting, token);
            }
        };
    }

    pub fn new(start: impl Into<BorrowedNode>) -> Self {
        Self {
            start: start.into(),
            mark: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<'arena, T> ChildScope<'arena, T> {
    pub fn add_child_by_ref<U>(&mut self, child_ref: OwnedNode<GenericASTNode>)
    where
        U: Into<GenericASTNode>,
        T: HasChildrenMarker<U>,
    {
        let token = &mut self.tree.stage.0;
        self.node_ref.rw(token).edges.push(child_ref.share());
        child_ref.rw(token).parent = Some(self.node_ref.share())
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
            self.add_child_by_ref(child_ref);
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
        let token = &self.tree.stage.0;
        NodeId::new(self.node_ref.ro(token).uid)
    }
}
