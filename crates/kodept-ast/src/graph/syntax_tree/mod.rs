use crate::graph::any_node::AnyNode;
use crate::graph::children::tags::{ChildTag, TAGS_DESC};
use crate::graph::node_id::NodeId;
use crate::graph::nodes::{NodeCell, PermTkn};
use crate::graph::syntax_tree::dfs::DfsIter;
use crate::graph::syntax_tree::stage::{
    CanAccess, CanMutAccess, FullAccess, ModificationAccess, NoAccess, ViewingAccess,
};
use crate::graph::utils::OptVec;
use crate::rlt_accessor::RLTAccessor;
use crate::traits::PopulateTree;
use kodept_core::structure::rlt::{File, RLT};
use kodept_core::structure::span::CodeHolder;
use kodept_core::{ConvertibleToMut, ConvertibleToRef};
use slotgraph::dag::{NodeKey, SecondaryDag};
use slotgraph::export::{Config, Dot};
use std::fmt::{Display, Formatter};

pub mod dfs;
pub mod stage;
pub mod subtree;

type Graph<T = NodeCell, E = ChildTag> = slotgraph::dag::Dag<T, E>;

#[derive(Debug)]
pub struct SyntaxTree<Permission = NoAccess> {
    inner: Graph,
    permission: Permission,
}

pub type SyntaxTreeBuilder = SyntaxTree<FullAccess>;
pub type SyntaxTreeView<'arena> = SyntaxTree<ViewingAccess<'arena>>;
pub type SyntaxTreeMutView<'arena> = SyntaxTree<ModificationAccess<'arena>>;

impl SyntaxTree<FullAccess> {
    pub fn export_dot<'a>(&'a self, config: &'a [Config]) -> impl Display + 'a {
        struct Helper<'a>(SecondaryDag<String, &'static str>, &'a [Config]);
        impl Display for Helper<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let dot = Dot::with_config(&self.0, self.1);
                write!(f, "{}", dot)
            }
        }

        Helper(
            self.inner.map(
                |k, v| format!("{} [{}]", v.ro(&self.permission.0).name(), k,),
                |tag| TAGS_DESC[*tag as usize],
            ),
            config,
        )
    }

    pub fn recursively_build<'a>(
        rlt_root: &'a RLT,
        context: &impl CodeHolder,
    ) -> (Self, RLTAccessor<'a>) {
        let subtree = rlt_root.0.convert(context);
        let (graph, accessor) = subtree.consume_map(NodeCell::new);
        let tree = Self {
            inner: graph,
            permission: FullAccess(PermTkn::new()),
        };
        (tree, accessor)
    }
    
    #[cfg(feature = "parallel")]
    pub fn parallel_recursively_build<'a>(
        rlt_root: &'a crate::traits::parallel::Parallelize<File, 0>,
        context: &impl CodeHolder,
    ) -> (Self, RLTAccessor<'a>) {
        let subtree = rlt_root.convert(context);
        let (graph, accessor) = subtree.consume_map(NodeCell::new);
        let tree = Self {
            inner: graph,
            permission: FullAccess(PermTkn::new())
        };
        (tree, accessor)
    }
}

impl SyntaxTree<NoAccess> {
    pub fn children_of<'b, T, U>(
        &'b self,
        id: NodeId<T>,
        token: &'b PermTkn,
        tag: ChildTag,
    ) -> OptVec<&'b U>
    where
        AnyNode: ConvertibleToRef<U>,
    {
        self.inner
            .children(id.into())
            .filter(|(_, it)| it.edge_data == tag)
            .filter_map(|(_, it)| it.value.ro(token).try_as_ref())
            .collect()
    }

    pub fn get<'b, T>(&'b self, id: NodeId<T>, token: &'b PermTkn) -> Option<&'b T>
    where
        AnyNode: ConvertibleToRef<T>,
    {
        let node_ref = self.inner.node_weight(id.into())?;
        node_ref.ro(token).try_as_ref()
    }

    pub fn get_mut<'b, T>(&'b self, id: NodeId<T>, token: &'b mut PermTkn) -> Option<&'b mut T>
    where
        AnyNode: ConvertibleToMut<T>,
    {
        let node_ref = self.inner.node_weight(id.into())?;
        node_ref.rw(token).try_as_mut()
    }

    pub fn parent_of<'b, T>(&'b self, id: NodeId<T>, token: &'b PermTkn) -> Option<&'b AnyNode> {
        let parent_id = self.inner.parent_id(id.into())?;
        Some(self.inner[parent_id].ro(token))
    }

    pub fn give_access(self, token: &PermTkn) -> SyntaxTreeView {
        SyntaxTree {
            inner: self.inner,
            permission: ViewingAccess(token),
        }
    }

    pub fn give_access_mut(self, token: &mut PermTkn) -> SyntaxTreeMutView {
        SyntaxTree {
            inner: self.inner,
            permission: ModificationAccess(token),
        }
    }
}

#[allow(private_bounds)]
impl<P: CanAccess> SyntaxTree<P> {
    pub fn children_of<T, U>(&self, id: NodeId<T>, tag: ChildTag) -> OptVec<&U>
    where
        AnyNode: ConvertibleToRef<U>,
    {
        let token = self.permission.tkn();
        self.inner
            .children(id.into())
            .filter(|(_, it)| it.edge_data == tag)
            .filter_map(|(_, it)| it.value.ro(token).try_as_ref())
            .collect()
    }

    pub fn get<T>(&self, id: NodeId<T>) -> Option<&T>
    where
        AnyNode: ConvertibleToRef<T>,
    {
        let token = self.permission.tkn();
        let node_ref = self.inner.node_weight(id.into())?;
        node_ref.ro(token).try_as_ref()
    }

    pub fn parent_of<T>(&self, id: NodeId<T>) -> Option<&AnyNode> {
        let token = self.permission.tkn();
        let node_ref = self.inner.parent_id(id.into())?;
        Some(self.inner[node_ref].ro(token))
    }
}

#[allow(private_bounds)]
impl<P: CanMutAccess> SyntaxTree<P> {
    pub fn get_mut<T>(&mut self, id: NodeId<T>) -> Option<&mut T>
    where
        AnyNode: ConvertibleToMut<T>,
    {
        let token = self.permission.tkn_mut();
        let node_ref = self.inner.node_weight(id.into())?;
        node_ref.rw(token).try_as_mut()
    }
}

impl<P> SyntaxTree<P> {
    pub fn node_count(&self) -> usize {
        self.inner.len()
    }

    pub fn dfs(&self) -> DfsIter<P> {
        DfsIter::new(self, NodeKey::Root)
    }

    pub fn raw_children_of<T>(&self, id: NodeId<T>, tag: ChildTag) -> OptVec<&NodeCell> {
        self.inner
            .children(id.into())
            .filter(|it| it.1.edge_data == tag)
            .map(|it| &it.1.value)
            .collect()
    }

    pub fn split(self) -> (SyntaxTree, P) {
        (
            SyntaxTree {
                inner: self.inner,
                permission: NoAccess,
            },
            self.permission,
        )
    }
}
