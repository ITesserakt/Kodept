use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

use kodept_core::{ConvertibleToMut, ConvertibleToRef, Named};
use petgraph::dot::{Config, Dot};
use petgraph::prelude::{NodeIndex, StableGraph};
use petgraph::{Directed, Direction};

use kodept_core::structure::span::CodeHolder;

use crate::graph::generic_node::{Node, NodeWithParent};
use crate::graph::nodes::{GhostToken, Owned, RefNode};
use crate::graph::utils::OptVec;
use crate::graph::{
    ChildTag, GenericASTNode, HasChildrenMarker, Identifiable, NodeId, DEFAULT_TAG,
};
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use crate::traits::{Linker, PopulateTree};
use crate::visitor::visit_side::VisitSide;

#[derive(Debug)]
pub struct BuildingStage(GhostToken);
#[derive(Default, Debug)]
pub struct AccessingStage;

type Graph<T = Owned, E = ChildTag> = StableGraph<T, E, Directed, usize>;

#[derive(Debug)]
pub struct SyntaxTree<Stage = AccessingStage> {
    graph: Graph,
    stage: Stage,
}

pub type SyntaxTreeBuilder = SyntaxTree<BuildingStage>;

pub struct ChildScope<'arena, T> {
    tree: &'arena mut SyntaxTree<BuildingStage>,
    node_id: NodeIndex<usize>,
    _phantom: PhantomData<T>,
}

#[derive(Clone)]
pub struct Dfs<'arena> {
    graph: &'arena Graph,
    start: NodeIndex<usize>,
}

impl Default for SyntaxTree<BuildingStage> {
    fn default() -> Self {
        // SAFE: While tree is building, token should be owned by it
        Self {
            graph: Default::default(),
            stage: BuildingStage(unsafe { GhostToken::new() }),
        }
    }
}

impl SyntaxTree<BuildingStage> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node<T>(&mut self, node: T) -> ChildScope<'_, T>
    where
        T: Into<GenericASTNode>,
    {
        let id = self.graph.add_node(Owned::new(node));
        let node_ref = &self.graph[id];
        node_ref.rw(&mut self.stage.0).set_id(id.into());

        ChildScope {
            tree: self,
            node_id: id,
            _phantom: PhantomData,
        }
    }

    pub fn build(self) -> SyntaxTree<AccessingStage> {
        SyntaxTree {
            graph: self.graph,
            stage: AccessingStage,
        }
    }

    pub fn export_dot<'a>(&'a self, config: &'a [Config]) -> impl Display + 'a {
        struct Wrapper<'c>(Graph<String, String>, &'c [Config]);
        impl Display for Wrapper<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let dot = Dot::with_config(&self.0, self.1);
                write!(f, "{dot}")
            }
        }

        let mapping = self.graph.map(
            |id, node| format!("{} [{}]", node.ro(&self.stage.0).name(), id.index()),
            |_, edge| {
                if *edge == DEFAULT_TAG {
                    "".to_string()
                } else {
                    format!("Tag = {edge}")
                }
            },
        );
        Wrapper(mapping, config)
    }
}

impl<U> SyntaxTree<U> {
    pub(crate) fn children_of_raw<T>(&self, id: NodeId<T>, tag: ChildTag) -> OptVec<&Owned> {
        let mut walker = self
            .graph
            .neighbors_directed(id.into(), Direction::Outgoing)
            .detach();
        let mut result = OptVec::Empty;
        while let Some((edge_id, node_id)) = walker.next(&self.graph) {
            if self.graph[edge_id] != tag {
                continue;
            }
            result.push(&self.graph[node_id])
        }
        result
    }
}

impl SyntaxTree {
    pub fn dfs(&self) -> Dfs {
        let mut roots = self.graph.externals(Direction::Incoming);
        let (Some(start), None) = (roots.next(), roots.next()) else {
            panic!("Syntax tree should have a root")
        };
        Dfs {
            graph: &self.graph,
            start,
        }
    }

    pub fn children_of<'b, T, U>(
        &'b self,
        id: NodeId<T>,
        token: &'b GhostToken,
        tag: ChildTag,
    ) -> OptVec<&U>
    where
        GenericASTNode: ConvertibleToRef<U>,
    {
        let mut walker = self
            .graph
            .neighbors_directed(id.into(), Direction::Outgoing)
            .detach();
        let mut result = OptVec::Empty;
        while let Some((edge_id, node_id)) = walker.next(&self.graph) {
            if self.graph[edge_id] != tag {
                continue;
            }
            if let Some(node) = self.graph[node_id].ro(token).try_as_ref() {
                result.push(node)
            }
        }
        result
    }

    pub fn get_mut<'b, T>(&'b self, id: NodeId<T>, token: &'b mut GhostToken) -> Option<&mut T>
    where
        GenericASTNode: ConvertibleToMut<T>,
    {
        let node_ref = self.graph.node_weight(id.into())?;
        node_ref.rw(token).try_as_mut()
    }

    pub fn parent_of<'a, T>(&'a self, id: NodeId<T>, token: &'a GhostToken) -> &T::Parent
    where
        T: NodeWithParent + Node,
        GenericASTNode: ConvertibleToRef<T::Parent>,
    {
        let mut parents = self
            .graph
            .neighbors_directed(id.into(), Direction::Incoming);
        let (Some(parent_id), None) = (parents.next(), parents.next()) else {
            panic!(
                "NodeWithParent contract violated: such kind of nodes should always have a parent"
            )
        };
        let parent_ref = self.graph[parent_id].ro(token);
        parent_ref.try_as_ref().expect("Node has wrong type")
    }

    pub fn parent_of_mut<'a, T>(
        &'a self,
        id: NodeId<T>,
        token: &'a mut GhostToken,
    ) -> &mut T::Parent
    where
        T: NodeWithParent + Node,
        GenericASTNode: ConvertibleToMut<T::Parent>,
    {
        let mut parents = self
            .graph
            .neighbors_directed(id.into(), Direction::Incoming);
        let (Some(parent_id), None) = (parents.next(), parents.next()) else {
            panic!(
                "NodeWithParent contract violated: such kind of nodes should always have a parent"
            )
        };
        let parent_ref = self.graph[parent_id].rw(token);
        parent_ref.try_as_mut().expect("Node has wrong type")
    }
}

impl<'arena> Dfs<'arena> {
    pub fn iter(&mut self, mut f: impl FnMut(RefNode<GenericASTNode>, VisitSide)) {
        self.dfs_impl(self.start, &mut |node, side| f(node, side));
    }

    fn dfs_impl(
        &mut self,
        current_id: NodeIndex<usize>,
        mut f: &mut dyn FnMut(RefNode<GenericASTNode>, VisitSide),
    ) {
        let Some(current) = self.graph.node_weight(current_id) else {
            return;
        };

        let mut edges = self
            .graph
            .neighbors_directed(current_id, Direction::Outgoing)
            .collect::<Vec<_>>()
            .into_iter()
            .peekable();
        if edges.peek().is_some() {
            f(current, VisitSide::Entering);
            for child in edges.rev() {
                self.dfs_impl(child, &mut f);
            }
            f(current, VisitSide::Exiting);
        } else {
            f(current, VisitSide::Leaf)
        }
    }
}

impl<'arena, T> ChildScope<'arena, T> {
    fn add_child_by_ref<U, const TAG: ChildTag>(&mut self, child_id: NodeIndex<usize>)
    where
        U: Into<GenericASTNode>,
        T: HasChildrenMarker<U, TAG>,
    {
        self.tree.graph.add_edge(self.node_id, child_id, TAG);
    }

    pub fn with_children_from<'b, const TAG: ChildTag, U: PopulateTree + 'b>(
        mut self,
        iter: impl IntoIterator<Item = &'b U>,
        context: &mut (impl Linker<'b> + CodeHolder),
    ) -> Self
    where
        T: HasChildrenMarker<U::Output, TAG>,
    {
        for item in iter {
            let child_id = item.convert(self.tree, context);
            self.add_child_by_ref(child_id.into());
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
        NodeId::new(self.node_id.index())
    }
}

impl From<NodeIndex<usize>> for NodeId<GenericASTNode> {
    fn from(value: NodeIndex<usize>) -> Self {
        NodeId::new(value.index())
    }
}

impl<T> From<NodeId<T>> for NodeIndex<usize> {
    fn from(value: NodeId<T>) -> Self {
        NodeIndex::new(value.into())
    }
}
