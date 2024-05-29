pub use main::{SyntaxTree, SyntaxTreeBuilder, SyntaxTreeMutView};
pub use utils::{ChildScope, DfsIter};

use crate::graph::nodes::Inaccessible;
use crate::graph::tags::ChildTag;

type Graph<T = Inaccessible, E = ChildTag> = slotgraph::DiGraph<T, E>;

mod stage {
    use crate::graph::PermTkn;

    #[derive(Debug)]
    pub struct FullAccess(pub(super) PermTkn);

    #[derive(Debug)]
    pub struct ViewingAccess<'a>(pub(super) &'a PermTkn);

    pub struct ModificationAccess<'a>(pub(super) &'a mut PermTkn);

    #[derive(Default, Debug)]
    pub struct NoAccess;

    pub(super) trait CanAccess {
        fn tkn(&self) -> &PermTkn;
    }

    impl CanAccess for FullAccess {
        fn tkn(&self) -> &PermTkn {
            &self.0
        }
    }

    impl CanAccess for ViewingAccess<'_> {
        fn tkn(&self) -> &PermTkn {
            self.0
        }
    }

    impl CanAccess for ModificationAccess<'_> {
        fn tkn(&self) -> &PermTkn {
            self.0
        }
    }
}

mod main {
    use std::fmt::{Display, Formatter};

    use kodept_core::{ConvertibleToMut, ConvertibleToRef};
    use slotgraph::export::{Config, Direction, Dot};
    use slotgraph::SubDiGraph;

    use crate::graph::{AnyNode, Change, ChangeSet, Identifiable, NodeId, PermTkn};
    use crate::graph::nodes::Inaccessible;
    use crate::graph::tags::{ChildTag, TAGS_DESC};
    use crate::graph::tree::{ChildScope, DfsIter, Graph};
    use crate::graph::tree::stage::{
        CanAccess, FullAccess, ModificationAccess, NoAccess, ViewingAccess,
    };
    use crate::graph::utils::OptVec;
    use crate::node_properties::{Node, NodeWithParent};
    use crate::Uninit;

    #[derive(Debug)]
    pub struct SyntaxTree<Permission = NoAccess> {
        pub(super) graph: Graph,
        pub(super) stage: Permission,
    }

    pub type SyntaxTreeBuilder = SyntaxTree<FullAccess>;
    pub type SyntaxTreeMutView<'token> = SyntaxTree<ModificationAccess<'token>>;

    impl Default for SyntaxTree<FullAccess> {
        fn default() -> Self {
            // SAFE: While tree is building, token should be owned by it
            Self {
                graph: Default::default(),
                stage: FullAccess(PermTkn::new()),
            }
        }
    }

    impl SyntaxTree<FullAccess> {
        pub fn new() -> SyntaxTree<FullAccess> {
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
                |id, node| format!("{} [{}]", node.ro(&self.stage.0).name(), id),
                |_, &edge| TAGS_DESC[edge as usize],
            );
            Wrapper(mapping, config)
        }
    }

    #[allow(private_bounds)]
    impl<U: CanAccess> SyntaxTree<U> {
        pub fn add_node<T>(&mut self, node: Uninit<T>) -> ChildScope<'_, T, U>
        where
            T: Identifiable + Into<AnyNode>,
        {
            let id = self
                .graph
                .add_node_with_key(|id| Inaccessible::new(node.unwrap(id.into())));
            ChildScope::new(self, id.into())
        }

        pub fn build(self) -> SyntaxTree<NoAccess> {
            SyntaxTree {
                graph: self.graph,
                stage: NoAccess,
            }
        }
    }

    impl SyntaxTree<ViewingAccess<'_>> {
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
                    let child_id = self
                        .graph
                        .add_node_with_key(|id| Inaccessible::new(child.unwrap(id.into())));
                    self.graph.add_edge(parent_id.into(), child_id, tag);
                }
                Change::Replace { from_id, to } => {
                    match self.graph.node_weight_mut(from_id.into()) {
                        None => {}
                        Some(x) => *x = Inaccessible::new(to.unwrap(from_id)),
                    };
                }
                Change::DeleteSelf { node_id } => {
                    self.graph.remove_node(node_id.into());
                }
            };
        }
    }

    impl<U> SyntaxTree<U> {
        pub(crate) fn children_of_raw<T>(
            &self,
            id: NodeId<T>,
            tag: ChildTag,
        ) -> OptVec<&Inaccessible> {
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
            token: &'b PermTkn,
            tag: ChildTag,
        ) -> OptVec<&U>
        where
            AnyNode: ConvertibleToRef<U>,
        {
            self.graph
                .children(id.into())
                .filter(|it| self.graph[it.0].data == tag)
                .filter_map(|it| self.graph[it.1].ro(token).try_as_ref())
                .collect()
        }

        pub fn get<'b, T>(&'b self, id: NodeId<T>, token: &'b PermTkn) -> Option<&T>
        where
            AnyNode: ConvertibleToRef<T>,
        {
            let node_ref = self.graph.node_weight(id.into())?;
            node_ref.ro(token).try_as_ref()
        }

        pub fn get_mut<'b, T>(&'b self, id: NodeId<T>, token: &'b mut PermTkn) -> Option<&mut T>
        where
            AnyNode: ConvertibleToMut<T>,
        {
            let node_ref = self.graph.node_weight(id.into())?;
            node_ref.rw(token).try_as_mut()
        }

        pub fn parent_of<'a, T>(&'a self, id: NodeId<T>, token: &'a PermTkn) -> Option<&AnyNode> {
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
            token: &'a mut PermTkn,
        ) -> &mut T::Parent
        where
            T: NodeWithParent + Node,
            AnyNode: ConvertibleToMut<T::Parent>,
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

        pub fn apply_changes(self, changes: ChangeSet, token: &PermTkn) -> Self {
            let mut this = self.modify(token);
            for change in changes {
                this.apply_change(change);
            }
            this.build()
        }

        fn modify(self, token: &PermTkn) -> SyntaxTree<ViewingAccess> {
            SyntaxTree {
                graph: self.graph,
                stage: ViewingAccess(token),
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

    use crate::graph::{AnyNode, HasChildrenMarker, NodeId, SyntaxTree};
    use crate::graph::nodes::Inaccessible;
    use crate::graph::tags::ChildTag;
    use crate::graph::tree::Graph;
    use crate::graph::tree::stage::{CanAccess, FullAccess};
    use crate::rlt_accessor::RLTFamily;
    use crate::traits::{Linker, PopulateTree};
    use crate::visit_side::VisitSide;

    pub struct ChildScope<'arena, T, Stage = FullAccess> {
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
        type Item = (&'arena Inaccessible, VisitSide);

        fn next(&mut self) -> Option<Self::Item> {
            let (next, descend) = self.stack.pop_back()?;
            let current = self.graph.node_weight(next)?;
            if matches!(descend, TraverseState::Exit) {
                return Some((current, VisitSide::Exiting));
            }

            self.edges_buffer.clear();
            self.edges_buffer
                .extend(self.graph.children(next).map(|it| it.1));
            self.edges_buffer.reverse();
            let edges_iter = self.edges_buffer.iter();
            if edges_iter.len() != 0 {
                self.stack.push_back((next, TraverseState::Exit));
                for child in edges_iter {
                    self.stack.push_back((*child, TraverseState::DescendDeeper));
                }
                Some((current, VisitSide::Entering))
            } else {
                Some((current, VisitSide::Leaf))
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.stack.len(), Some(self.graph.node_count() * 2))
        }
    }

    impl FusedIterator for DfsIter<'_> {}

    impl<'arena, T, S> ChildScope<'arena, T, S> {
        pub(super) fn new(tree: &'arena mut SyntaxTree<S>, node_id: Key<T>) -> Self {
            Self { tree, id: node_id }
        }

        fn add_child_by_ref<U, const TAG: ChildTag>(&mut self, child_id: NodeKey)
        where
            U: Into<AnyNode>,
            T: HasChildrenMarker<U, TAG>,
        {
            self.tree.graph.add_edge(self.id.into(), child_id, TAG);
        }

        #[allow(private_bounds)]
        pub fn with_rlt<U>(self, context: &mut impl Linker, rlt_node: &U) -> Self
        where
            U: Into<RLTFamily> + Clone,
            T: Into<AnyNode>,
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

    impl<T> ChildScope<'_, T, FullAccess> {
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

mod subtree {
    use crate::graph::AnyNode;
    use crate::graph::tree::Graph;

    #[derive(Default, Debug)]
    pub struct SubSyntaxTree {
        graph: Graph<AnyNode>,
    }

    impl SubSyntaxTree {
        pub fn new() -> Self {
            Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FileDecl;
    use crate::graph::SyntaxTreeBuilder;

    #[test]
    fn test_tree_creation() {
        let mut builder = SyntaxTreeBuilder::new();

        let id = builder.add_node(FileDecl::uninit()).id();

        let tree = builder.build();
        let children = tree.children_of_raw(id, 0);

        assert!(children.is_empty());
    }
}
