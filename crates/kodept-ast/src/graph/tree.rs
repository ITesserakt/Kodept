pub use stage::*;
pub use tree::*;
pub use utils::*;

use crate::graph::nodes::Owned;
use crate::graph::tags::ChildTag;

type Graph<T = Owned, E = ChildTag> = slotgraph::DiGraph<T, E>;

mod stage {
    use crate::graph::GhostToken;

    #[derive(Debug)]
    pub struct BuildingStage(pub(super) GhostToken);

    #[derive(Debug)]
    pub struct ModifyingStage<'a>(pub(super) &'a GhostToken);

    #[derive(Default, Debug)]
    pub struct AccessingStage;

    pub(super) trait CanAccess {
        fn tkn(&self) -> &GhostToken;
    }

    impl CanAccess for BuildingStage {
        fn tkn(&self) -> &GhostToken {
            &self.0
        }
    }

    impl CanAccess for ModifyingStage<'_> {
        fn tkn(&self) -> &GhostToken {
            self.0
        }
    }
}

mod tree {
    use std::fmt::{Display, Formatter};

    use kodept_core::{ConvertibleToMut, ConvertibleToRef};
    use slotgraph::export::{Config, Direction, Dot};
    use slotgraph::SubDiGraph;

    use crate::graph::{Change, ChangeSet, GenericASTNode, GhostToken, Identifiable, NodeId};
    use crate::graph::nodes::Owned;
    use crate::graph::tags::{ChildTag, TAGS_DESC};
    use crate::graph::tree::{AccessingStage, BuildingStage, ChildScope, DfsIter, Graph, ModifyingStage};
    use crate::graph::tree::stage::CanAccess;
    use crate::graph::utils::OptVec;
    use crate::node_properties::{Node, NodeWithParent};

    #[derive(Debug)]
    pub struct SyntaxTree<Stage = AccessingStage> {
        pub(super) graph: Graph,
        pub(super) stage: Stage,
    }

    pub type SyntaxTreeBuilder = SyntaxTree<BuildingStage>;

    impl Default for SyntaxTree<BuildingStage> {
        fn default() -> Self {
            // SAFE: While tree is building, token should be owned by it
            Self {
                graph: Default::default(),
                stage: BuildingStage(GhostToken::new()),
            }
        }
    }

    impl SyntaxTree<BuildingStage> {
        pub fn new() -> SyntaxTree<BuildingStage> {
            SyntaxTree::default()
        }

        pub fn export_dot<'a>(&'a self, config: &'a [Config]) -> impl Display + 'a {
            struct Wrapper<'c>(SubDiGraph<String, &'static str>, &'c [Config]);
            impl Display for Wrapper<'_> {
                fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                    let dot = Dot::with_config(&self.0, self.1);
                    write!(f, "{dot}")
                }
            }

            let mapping = self.graph.map(
                |id, node| format!("{} [{:?}]", node.ro(&self.stage.0).name(), id),
                |_, &edge| TAGS_DESC[edge as usize],
            );
            Wrapper(mapping, config)
        }
    }

    #[allow(private_bounds)]
    impl<U: CanAccess> SyntaxTree<U> {
        pub fn add_node<T>(&mut self, node: T) -> ChildScope<'_, T, U>
            where
                T: Into<GenericASTNode>,
        {
            let id = self.graph.add_node(Owned::new(node));
            let node_ref = &self.graph[id];
            node_ref.ro(self.stage.tkn()).set_id(id.into());

            ChildScope::new(self, id.into())
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
                    self.graph[id].ro(self.stage.tkn()).set_id(id.into())
                }
                Change::Replace { from_id, to } => {
                    match self.graph.node_weight_mut(from_id.into()) {
                        None => {}
                        Some(x) => {
                            *x = Owned::new(to);
                            x.ro(self.stage.tkn()).set_id(from_id);
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
            self.graph
                .children(id.into())
                .filter(|it| self.graph[it.0].data == tag)
                .map(|it| &self.graph[it.1])
                .collect()
        }

        pub fn dfs(&self) -> DfsIter {
            let mut roots = self.graph.externals(Direction::Incoming);
            let (Some(start), None) = (roots.next(), roots.next()) else {
                panic!("Syntax tree should have a root")
            };
            
            DfsIter::new(&self.graph, start)
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
            self.graph
                .children(id.into())
                .filter(|it| self.graph[it.0].data == tag)
                .filter_map(|it| self.graph[it.1].ro(token).try_as_ref())
                .collect()
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
            let mut parents = self.graph.parents(id.into());
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
            let mut parents = self.graph.parents(id.into());
            let (Some((_, parent_id)), None) = (parents.next(), parents.next()) else {
                panic!(
                    "NodeWithParent contract violated: such kind of nodes should always have a parent"
                )
            };
            let parent_ref = self.graph[parent_id].rw(token);
            parent_ref.try_as_mut().expect("Node has wrong type")
        }

        pub fn apply_changes(self, changes: ChangeSet, token: &GhostToken) -> Self {
            let mut this = self.modify(token);
            for change in changes {
                this.apply_change(change);
            }
            this.build()
        }

        fn modify(self, token: &GhostToken) -> SyntaxTree<ModifyingStage> {
            SyntaxTree {
                graph: self.graph,
                stage: ModifyingStage(token),
            }
        }
    }
}

mod utils {
    use std::collections::VecDeque;
    use std::iter::FusedIterator;

    use kodept_core::structure::span::CodeHolder;
    use slotgraph::{Key, NodeKey};
    use slotgraph::export::NodeCount;

    use crate::graph::{GenericASTNode, HasChildrenMarker, NodeId, SyntaxTree};
    use crate::graph::nodes::Owned;
    use crate::graph::tags::ChildTag;
    use crate::graph::tree::{BuildingStage, Graph};
    use crate::graph::tree::stage::CanAccess;
    use crate::rlt_accessor::RLTFamily;
    use crate::traits::{Linker, PopulateTree};
    use crate::visit_side::VisitSide;

    pub struct ChildScope<'arena, T, Stage = BuildingStage> {
        tree: &'arena mut SyntaxTree<Stage>,
        id: Key<T>,
    }

    pub enum TraverseState {
        DescendDeeper,
        Exit,
    }

    pub struct DfsIter<'a> {
        stack: VecDeque<(NodeKey, TraverseState)>,
        edges_buffer: Vec<NodeKey>,
        graph: &'a Graph,
    }

    impl<'arena> DfsIter<'arena> {
        pub(super) fn new(graph: &'arena Graph, start: NodeKey) -> Self {
            let mut stack = VecDeque::with_capacity(graph.node_count());
            stack.push_back((start, TraverseState::DescendDeeper));
            
            Self {
                stack,
                edges_buffer: vec![],
                graph,
            }
        }
    }

    impl<'arena> Iterator for DfsIter<'arena> {
        type Item = (&'arena Owned, VisitSide);

        fn next(&mut self) -> Option<Self::Item> {
            let Some((next, descend)) = self.stack.pop_back() else {
                return None;
            };
            let Some(current) = self.graph.node_weight(next) else {
                return None;
            };
            if matches!(descend, TraverseState::Exit) {
                return Some((current, VisitSide::Exiting));
            }

            self.edges_buffer.clear();
            self.edges_buffer
                .extend(self.graph.children(next).map(|it| it.1));
            self.edges_buffer.reverse();
            let edges_iter = self.edges_buffer.iter();
            return if edges_iter.len() != 0 {
                self.stack.push_back((next, TraverseState::Exit));
                for child in edges_iter {
                    self.stack.push_back((*child, TraverseState::DescendDeeper));
                }
                Some((current, VisitSide::Entering))
            } else {
                Some((current, VisitSide::Leaf))
            };
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.stack.len(), Some(self.graph.node_count() * 2))
        }
    }

    impl FusedIterator for DfsIter<'_> {}

    impl<'arena, T, S> ChildScope<'arena, T, S> {
        pub(super) fn new(tree: &'arena mut SyntaxTree<S>, node_id: Key<T>) -> Self {
            Self {
                tree,
                id: node_id,
            }
        }
        
        fn add_child_by_ref<U, const TAG: ChildTag>(&mut self, child_id: NodeKey)
            where
                U: Into<GenericASTNode>,
                T: HasChildrenMarker<U, TAG>,
        {
            self.tree.graph.add_edge(self.id.into(), child_id, TAG);
        }

        #[allow(private_bounds)]
        pub fn with_rlt<U>(self, context: &mut impl Linker, rlt_node: &U) -> Self
            where
                U: Into<RLTFamily> + Clone,
                T: Into<GenericASTNode>,
                S: CanAccess,
        {
            let element = &self.tree.graph[NodeKey::from(self.id)];
            let node = element.ro(self.tree.stage.tkn());

            context.link(node, rlt_node);
            self
        }

        pub fn id(&self) -> NodeId<T> {
            self.id.into()
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
}

#[cfg(test)]
mod tests {
    use crate::FileDeclaration;
    use crate::graph::SyntaxTreeBuilder;

    #[test]
    fn test_tree_creation() {
        let mut builder = SyntaxTreeBuilder::new();
        
        let id = builder.add_node(FileDeclaration::uninit()).id();
        
        let tree = builder.build();
        let children = tree.children_of_raw(id, 0);
        
        assert!(children.is_empty());
    }
}