use crate::graph::children::tags::ChildTag;
use crate::graph::children::HasChildrenMarker;
use crate::graph::node_id::GenericNodeKey;
use crate::graph::syntax_tree::utils;
use crate::graph::syntax_tree::Graph;
use crate::graph::{AnyNode, Identifiable, NodeId};
use crate::rlt_accessor::{RLTAccessor, RLTFamily};
use crate::traits::PopulateTree;
use crate::uninit::Uninit;
use kodept_core::structure::span::CodeHolder;
use replace_with::replace_with_or_abort_and_return;
use slotgraph::dag::{Dag, NodeKey};
use slotmap::{Key, SecondaryMap};
use std::convert::identity;
use std::marker::PhantomData;
use std::sync::LazyLock;

static SWITCH_TO_PARALLEL_THRESHOLD: LazyLock<usize> =
    LazyLock::new(|| match std::thread::available_parallelism() {
        Ok(x) => x.get() * 4,
        Err(_) => 8 * 4,
    });

#[derive(Debug)]
enum GraphImpl {
    Plain(Graph<AnyNode>),
    Leaf { root: AnyNode },
}

#[derive(Debug)]
pub struct SubSyntaxTree<'rlt, ROOT> {
    graph: GraphImpl,
    rlt_mapping: SecondaryMap<GenericNodeKey, RLTFamily<'rlt>>,
    root_rlt_mapping: Option<RLTFamily<'rlt>>,
    _phantom: PhantomData<ROOT>,
}

impl<'rlt, T> SubSyntaxTree<'rlt, T> {
    pub(super) fn from_dag(graph: Graph<AnyNode>) -> SubSyntaxTree<'static, T> {
        SubSyntaxTree {
            graph: GraphImpl::Plain(graph),
            rlt_mapping: Default::default(),
            root_rlt_mapping: None,
            _phantom: Default::default(),
        }
    }

    #[allow(private_bounds)]
    pub fn new(root: Uninit<'rlt, T>) -> Self
    where
        T: Into<AnyNode> + Identifiable,
    {
        let (root, mapping) = root.unwrap(NodeId::Root);
        Self {
            graph: GraphImpl::Leaf { root: root.into() },
            rlt_mapping: Default::default(),
            root_rlt_mapping: mapping,
            _phantom: PhantomData,
        }
    }

    #[allow(private_bounds)]
    pub fn add_child<U, const TAG: ChildTag>(&mut self, node: Uninit<'rlt, U>) -> NodeId<U>
    where
        T: HasChildrenMarker<U, TAG>,
        U: Identifiable + Into<AnyNode>,
    {
        let mut rlt = None;
        let id = replace_with_or_abort_and_return(&mut self.graph, |g| {
            let mut g = match g {
                GraphImpl::Leaf { root } => Graph::new(root),
                GraphImpl::Plain(g) => g,
            };
            let id = g.add_node_at_root(|id| {
                let node = node.unwrap(id.into());
                rlt = node.1;
                (node.0.into(), TAG)
            });
            (id, GraphImpl::Plain(g))
        });
        if let (Some(rlt), NodeKey::Child(id)) = (rlt, id) {
            self.rlt_mapping.insert(id.into(), rlt);
        }
        id.into()
    }

    pub fn attach_subtree<U, const TAG: ChildTag>(&mut self, subtree: SubSyntaxTree<'rlt, U>)
    where
        T: HasChildrenMarker<U, TAG>,
    {
        let (id, mapping) = replace_with_or_abort_and_return(&mut self.graph, |g| {
            let mut g = match g {
                GraphImpl::Plain(g) => g,
                GraphImpl::Leaf { root } => Graph::new(root),
            };
            let result = match subtree.graph {
                GraphImpl::Plain(sg) => {
                    let (id, mapping) = g.attach_subgraph_at(NodeKey::Root, sg, TAG).unwrap();
                    for &to in mapping.values() {
                        g[to].set_id(to.into());
                    }
                    (id, mapping)
                }
                GraphImpl::Leaf { root } => {
                    let id = g.add_node_at_root(|id| {
                        root.set_id(id.into());
                        (root, TAG)
                    });
                    (id, Default::default())
                }
            };
            (result, GraphImpl::Plain(g))
        });
        if let (NodeKey::Child(id), Some(map)) = (id, subtree.root_rlt_mapping) {
            self.rlt_mapping.insert(id.into(), map);
        }
        for (k, v) in subtree.rlt_mapping.into_iter() {
            let NodeKey::Child(mapped_key) = mapping[&NodeKey::Child(k.data().into())] else {
                continue;
            };
            self.rlt_mapping.insert(mapped_key.into(), v);
        }
    }

    pub fn maybe_with_children_from<'a: 'rlt, const TAG: ChildTag, U>(
        self,
        from: Option<
            impl utils::IntoCommonIter<Item = impl PopulateTree<'a, Root = U>> + utils::HasLength,
        >,
        context: &impl CodeHolder,
    ) -> Self
    where
        T: HasChildrenMarker<U, TAG> + Send,
        U: Send,
    {
        if let Some(from) = from {
            return self.with_children_from(from, context);
        }
        self
    }

    pub fn with_children_from<'a: 'rlt, const TAG: ChildTag, U>(
        mut self,
        iter: impl utils::IntoCommonIter<Item = impl PopulateTree<'a, Root = U>> + utils::HasLength,
        context: &impl CodeHolder,
    ) -> Self
    where
        U: Send,
        T: HasChildrenMarker<U, TAG> + Send,
    {
        if !cfg!(feature = "parallel") || iter.len() < *SWITCH_TO_PARALLEL_THRESHOLD {
            return iter
                .into_iter()
                .map(|it| it.convert(context))
                .fold(self, |mut acc, next| {
                    acc.attach_subtree(next);
                    acc
                });
        }

        #[cfg(not(feature = "parallel"))]
        unreachable!();

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;

            let (sx, rx) = std::sync::mpsc::channel();
            let iter = iter.into_par_iter();

            let (_, result) = rayon::join(
                move || {
                    iter.map(|it| it.convert(context))
                        .for_each_with(sx, |sender, it| sender.send(it).unwrap());
                },
                move || {
                    for item in rx {
                        self.attach_subtree(item)
                    }
                    self
                },
            );
            result
        }
    }

    pub fn cast<R>(self) -> SubSyntaxTree<'rlt, R>
    where
        T: Into<R>,
    {
        SubSyntaxTree {
            graph: self.graph,
            rlt_mapping: self.rlt_mapping,
            root_rlt_mapping: self.root_rlt_mapping,
            _phantom: PhantomData,
        }
    }

    pub fn extract_children<U, const TAG: ChildTag>(
        &mut self,
    ) -> impl Iterator<Item = SubSyntaxTree<'rlt, U>> + '_
    where
        T: HasChildrenMarker<U, TAG>,
        U: 'static
    {
        enum Helper<I1, I2> {
            A(I1),
            B(I2),
        }
        impl<T, I1, I2> Iterator for Helper<I1, I2>
        where
            I1: Iterator<Item = T>,
            I2: Iterator<Item = T>,
        {
            type Item = T;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Helper::A(x) => x.next(),
                    Helper::B(x) => x.next(),
                }
            }
        }

        match &mut self.graph {
            GraphImpl::Plain(g) => {
                let children_ids: Vec<_> = g
                    .children(NodeKey::Root)
                    .filter(|(_, it)| it.edge_data == TAG)
                    .map(|(id, _)| id)
                    .collect();

                Helper::A(
                    children_ids
                        .into_iter()
                        .map(|id| g.detach_subgraph_at(id))
                        .filter_map(|it| Some(SubSyntaxTree::from_dag(it?.0))),
                )
            }
            GraphImpl::Leaf { .. } => Helper::B(std::iter::empty()),
        }
    }
    
    pub fn into_root(self) -> Uninit<'rlt, T>
    where 
        AnyNode: TryInto<T>
    {
        let root = match self.graph {
            GraphImpl::Plain(g) => g.root,
            GraphImpl::Leaf { root } => root,
        };
        
        let value = if let Some(rlt) = self.root_rlt_mapping {
            Uninit::new(root)
                .with_rlt(rlt)
        } else {
            Uninit::new(root)
        };
        
        value.map(|it| it.try_into().ok().unwrap())
    } 

    pub(super) fn consume_map<U>(
        self,
        mut f: impl FnMut(AnyNode) -> U,
    ) -> (Dag<U, ChildTag>, RLTAccessor<'rlt>) {
        let accessor = RLTAccessor::new(self.rlt_mapping, self.root_rlt_mapping);
        match self.graph {
            GraphImpl::Plain(g) => (g.consume_map(f, identity), accessor),
            GraphImpl::Leaf { root } => (Dag::new(f(root)), accessor),
        }
    }
}
