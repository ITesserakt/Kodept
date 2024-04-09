use std::fmt::{Debug, Display, Formatter};
use std::iter;
use std::marker::PhantomData;
use std::ops::Coroutine;

use daggy::petgraph::dot::{Config, Dot};
use daggy::petgraph::Direction;
use daggy::stable_dag::StableDag;
use daggy::{NodeIndex, Walker};

use kodept_core::structure::span::CodeHolder;
use kodept_core::{ConvertibleTo, ConvertibleToMut, ConvertibleToRef};

use crate::graph::generic_node::{Node, NodeWithParent};
use crate::graph::nodes::{GhostToken, Owned, RefNode};
use crate::graph::tags::{ChildTag, TAGS_DESC};
use crate::graph::utils::OptVec;
use crate::graph::{Change, ChangeSet, GenericASTNode, HasChildrenMarker, Identifiable, NodeId};
use crate::rlt_accessor::{ASTFamily, RLTFamily};
use crate::traits::{Linker, PopulateTree};
use crate::visit_side::VisitSide;
use crate::yield_all;

#[derive(Debug)]
pub struct BuildingStage(GhostToken);

#[derive(Debug)]
pub struct ModifyingStage<'a>(&'a mut GhostToken);

#[derive(Default, Debug)]
pub struct AccessingStage;

trait CanModify {
    fn tkn(&mut self) -> &mut GhostToken;
}

impl CanModify for BuildingStage {
    fn tkn(&mut self) -> &mut GhostToken {
        &mut self.0
    }
}

impl CanModify for ModifyingStage<'_> {
    fn tkn(&mut self) -> &mut GhostToken {
        self.0
    }
}

type Graph<T = Owned, E = ChildTag> = StableDag<T, E, usize>;

#[derive(Debug)]
pub struct SyntaxTree<Stage = AccessingStage> {
    graph: Graph,
    stage: Stage,
}

pub type SyntaxTreeBuilder = SyntaxTree<BuildingStage>;

pub struct ChildScope<'arena, T, Stage = BuildingStage> {
    tree: &'arena mut SyntaxTree<Stage>,
    node_id: NodeIndex<usize>,
    _phantom: PhantomData<T>,
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
    pub fn new() -> SyntaxTree<BuildingStage> {
        SyntaxTree::default()
    }

    pub fn export_dot<'a>(&'a self, config: &'a [Config]) -> impl Display + 'a {
        struct Wrapper<'c>(Graph<String, &'static str>, &'c [Config]);
        impl Display for Wrapper<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let dot = Dot::with_config(&self.0, self.1);
                write!(f, "{dot}")
            }
        }

        let mapping = self.graph.map(
            |id, node| format!("{} [{}]", node.ro(&self.stage.0).name(), id.index()),
            |_, &edge| TAGS_DESC[edge as usize],
        );
        Wrapper(mapping, config)
    }
}

#[allow(private_bounds)]
impl<U: CanModify> SyntaxTree<U> {
    pub fn add_node<T>(&mut self, node: T) -> ChildScope<'_, T, U>
    where
        T: Into<GenericASTNode>,
    {
        let id = self.graph.add_node(Owned::new(node));
        let node_ref = &self.graph[id];
        node_ref.rw(self.stage.tkn()).set_id(id.into());

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
}

impl SyntaxTree<ModifyingStage<'_>> {
    fn apply_change(&mut self, change: Change) {
        match change {
            Change::Delete { child_id, .. } => {
                self.graph.remove_node(child_id.into());
            }
            Change::Add {
                parent_id,
                child,
                tag,
            } => {
                let (_, id) = self
                    .graph
                    .add_child(parent_id.into(), tag, Owned::new(child));
                self.graph[id].rw(self.stage.0).set_id(id.into())
            }
            Change::Replace { from_id, to } => {
                match self.graph.node_weight_mut(from_id.into()) {
                    None => {}
                    Some(x) => {
                        *x = Owned::new(to);
                        x.rw(self.stage.tkn()).set_id(from_id);
                    }
                };
            }
            Change::DeleteSelf { node_id } => {
                self.graph.remove_node(node_id.into());
            }
        };
    }
}

impl<U> SyntaxTree<U> {
    pub(crate) fn children_of_raw<T>(&self, id: NodeId<T>, tag: ChildTag) -> OptVec<&Owned> {
        let mut walker = self.graph.children(id.into());
        let mut result = OptVec::new();
        while let Some((edge_id, node_id)) = walker.walk_next(&self.graph) {
            if self.graph[edge_id] != tag {
                continue;
            }
            result.push(&self.graph[node_id])
        }
        result
    }

    pub fn dfs(&self) -> impl Iterator<Item = (RefNode, VisitSide)> + '_ {
        let mut roots = self.graph.graph().externals(Direction::Incoming);
        let (Some(start), None) = (roots.next(), roots.next()) else {
            panic!("Syntax tree should have a root")
        };
        iter::from_coroutine(move || yield_all!(coroutine(&self.graph, start)))
    }
}

impl SyntaxTree {
    pub fn children_of<'b, T, U>(
        &'b self,
        id: NodeId<T>,
        token: &'b GhostToken,
        tag: ChildTag,
    ) -> OptVec<&U>
    where
        GenericASTNode: ConvertibleToRef<U>,
    {
        let mut walker = self.graph.children(id.into());
        let mut result = OptVec::new();
        while let Some((edge_id, node_id)) = walker.walk_next(&self.graph) {
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

    pub fn parent_of<'a, T>(
        &'a self,
        id: NodeId<T>,
        token: &'a GhostToken,
    ) -> Option<&GenericASTNode> {
        let mut parents = self.graph.parents(id.into()).iter(&self.graph);
        if let (Some((_, parent_id)), None) = (parents.next(), parents.next()) {
            Some(self.graph[parent_id].ro(token))
        } else {
            None
        }
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
        let mut parents = self.graph.parents(id.into()).iter(&self.graph);
        let (Some((_, parent_id)), None) = (parents.next(), parents.next()) else {
            panic!(
                "NodeWithParent contract violated: such kind of nodes should always have a parent"
            )
        };
        let parent_ref = self.graph[parent_id].rw(token);
        parent_ref.try_as_mut().expect("Node has wrong type")
    }

    pub fn apply_changes(self, changes: ChangeSet, token: &mut GhostToken) -> Self {
        let mut this = self.modify(token);
        for change in changes {
            this.apply_change(change);
        }
        this.build()
    }

    fn modify(self, token: &mut GhostToken) -> SyntaxTree<ModifyingStage> {
        SyntaxTree {
            graph: self.graph,
            stage: ModifyingStage(token),
        }
    }
}

fn coroutine(
    graph: &Graph,
    start: NodeIndex<usize>,
) -> Box<dyn Coroutine<Return = (), Yield = (RefNode, VisitSide)> + '_> {
    Box::new(move || {
        let Some(current) = graph.node_weight(start) else {
            return;
        };
        let mut edges = graph.children(start).iter(graph).peekable();
        if edges.peek().is_some() {
            yield (current, VisitSide::Entering);
            for (_, child) in edges.collect::<Vec<_>>().into_iter().rev() {
                yield_all!(coroutine(graph, child));
            }
            yield (current, VisitSide::Exiting);
        } else {
            yield (current, VisitSide::Leaf);
        }
    })
}

impl<'arena, T, S> ChildScope<'arena, T, S> {
    fn add_child_by_ref<U, const TAG: ChildTag>(&mut self, child_id: NodeIndex<usize>)
    where
        U: Into<GenericASTNode>,
        T: HasChildrenMarker<U, TAG>,
    {
        self.tree
            .graph
            .add_edge(self.node_id, child_id, TAG)
            .expect("Given layout don't form DAG");
    }

    pub fn with_rlt<U>(self, context: &mut impl Linker, rlt_node: &U) -> Self
    where
        U: Into<RLTFamily> + Clone,
        NodeId<T>: Into<ASTFamily>,
    {
        context.link_ref(self.id(), rlt_node);
        self
    }

    pub fn id(&self) -> NodeId<T> {
        NodeId::new(self.node_id.index())
    }

    pub fn cast<U>(self) -> ChildScope<'arena, U, S>
    where
        T: ConvertibleTo<U>,
    {
        ChildScope {
            tree: self.tree,
            node_id: self.node_id,
            _phantom: Default::default(),
        }
    }
}

impl<T> ChildScope<'_, T, BuildingStage> {
    #[allow(private_bounds)]
    pub fn with_children_from<'b, const TAG: ChildTag, U>(
        mut self,
        iter: impl IntoIterator<Item = &'b U>,
        context: &mut (impl Linker + CodeHolder),
    ) -> Self
    where
        T: HasChildrenMarker<U::Output, TAG>,
        U: PopulateTree + 'b,
    {
        for item in iter {
            let child_id = item.convert(self.tree, context);
            self.add_child_by_ref(child_id.into());
        }
        self
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
