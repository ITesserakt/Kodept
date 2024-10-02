use crate::graph::any_node::AnyNode;
use crate::graph::children::tags::{ChildTag, TAGS_DESC};
use crate::graph::node_id::NodeId;
use crate::graph::syntax_tree::dfs::DfsIter;
use crate::graph::utils::OptVec;
use crate::graph::{HasChildrenMarker, SubSyntaxTree};
use crate::rlt_accessor::RLTAccessor;
use crate::traits::PopulateTree;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
use kodept_core::{ConvertibleToMut, ConvertibleToRef};
use slotgraph::dag::{SecondaryDag};
use slotgraph::export::{Config, Dot};
use std::convert::identity;
use std::fmt::{Display, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use crate::interning::SharedStr;

pub mod dfs;
pub(crate) mod subtree;
mod utils;

type Graph<T = AnyNode, E = ChildTag> = slotgraph::dag::Dag<T, E>;

#[derive(Debug)]
pub struct SyntaxTree<Permission = ()> {
    inner: Graph,
    pub(crate) permission: PhantomData<Permission>,
}

pub type SyntaxTreeBuilder = SyntaxTree<()>;

impl<P> SyntaxTree<P> {
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
                |k, v| format!("{} [{}]", v.name(), k,),
                |tag| TAGS_DESC[*tag as usize],
            ),
            config,
        )
    }

    pub fn recursively_build(
        rlt_root: &rlt::RLT,
        context: impl CodeHolder<Str=SharedStr>,
    ) -> (Self, RLTAccessor) {
        let subtree = rlt_root.0.convert(context);
        let (graph, accessor) = subtree.consume_map(identity);
        let tree = Self {
            inner: graph,
            permission: Default::default(),
        };
        (tree, accessor)
    }

    pub fn detach_subtree<T>(
        &mut self,
        node_id: NodeId<T>,
    ) -> Option<(SubSyntaxTree<'static, T>, ChildTag)> {
        let (dag, edge) = self.inner.detach_subgraph_at(node_id.into())?;
        Some((SubSyntaxTree::from_dag(dag), edge))
    }

    pub fn attach_subtree<T, U>(
        &mut self,
        parent_id: NodeId<T>,
        subtree: SubSyntaxTree<U>,
        tag: ChildTag,
    ) {
        let (dag, _) = subtree.consume_map(identity);
        self.inner
            .attach_subgraph_at(parent_id.into(), dag, tag)
            .unwrap();
    }

    pub fn update_children_tag<T1, U1, T2, U2, const OLD_TAG: ChildTag, const NEW_TAG: ChildTag>(
        &mut self,
        node_id: NodeId<T1>,
    ) -> Option<()>
    where
        T1: HasChildrenMarker<U1, OLD_TAG>,
        T2: HasChildrenMarker<U2, NEW_TAG>,
    {
        for id in self.inner.children_ids(node_id.into()).collect::<Vec<_>>() {
            let Some(weight) = self.inner.edge_weight_mut(id) else {
                continue;
            };
            if *weight == OLD_TAG {
                *weight = NEW_TAG;
            }
        }
        Some(())
    }

    pub fn add_child<T, U>(
        &mut self,
        parent_id: NodeId<T>,
        f: impl FnOnce(NodeId<U>) -> U,
        tag: ChildTag,
    ) -> NodeId<U>
    where
        AnyNode: From<U>,
    {
        self.inner
            .add_node_with_key(parent_id.into(), |id| (f(id.into()).into(), tag))
            .unwrap()
            .into()
    }

    pub fn children_of<'a, T, U>(
        &'a self,
        id: NodeId<T>,
        tag: ChildTag,
    ) -> impl FusedIterator<Item = &'a U>
    where
        AnyNode: ConvertibleToRef<U>,
        U: 'a,
    {
        self.inner
            .children(id.into())
            .filter(move |(_, it)| it.edge_data == tag)
            .filter_map(|(_, it)| it.value.try_as_ref())
    }
    
    pub fn contains<T>(&self, id: NodeId<T>) -> bool {
        self.inner.contains(id.into())
    }

    pub fn get<T>(&self, id: NodeId<T>) -> Option<&T>
    where
        AnyNode: ConvertibleToRef<T>,
    {
        let node_ref = self.inner.node_weight(id.into())?;
        node_ref.try_as_ref()
    }

    pub fn parent_of<T>(&self, id: NodeId<T>) -> Option<&AnyNode> {
        let node_ref = self.inner.parent_id(id.into())?;
        Some(&self.inner[node_ref])
    }

    pub fn get_mut<T>(&mut self, id: NodeId<T>) -> Option<&mut T>
    where
        AnyNode: ConvertibleToMut<T>,
    {
        let node_ref = self.inner.node_weight_mut(id.into())?;
        node_ref.try_as_mut()
    }

    pub fn node_count(&self) -> usize {
        self.inner.len()
    }

    pub fn dfs(&self) -> DfsIter<P> {
        DfsIter::new(self, NodeId::Root)
    }

    pub fn raw_children_of<T>(&self, id: NodeId<T>, tag: ChildTag) -> OptVec<&AnyNode> {
        self.inner
            .children(id.into())
            .filter(|it| it.1.edge_data == tag)
            .map(|it| &it.1.value)
            .collect()
    }
}
