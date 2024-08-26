use crate::graph::children::tags::ChildTag;
use crate::graph::children::HasChildrenMarker;
use crate::graph::node_id::GenericNodeKey;
use crate::graph::syntax_tree::Graph;
use crate::graph::{AnyNode, Identifiable, NodeId};
use crate::rlt_accessor::RLTFamily;
use crate::traits::PopulateTree;
use crate::uninit::Uninit;
use kodept_core::structure::span::CodeHolder;
use slotgraph::dag::NodeKey;
use slotmap::{Key, SecondaryMap};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct SubSyntaxTree<'rlt, ROOT> {
    pub(super) graph: Graph<AnyNode>,
    pub(super) rlt_mapping: SecondaryMap<GenericNodeKey, RLTFamily<'rlt>>,
    pub(super) root_rlt_mapping: Option<RLTFamily<'rlt>>,
    _phantom: PhantomData<ROOT>,
}

impl<'rlt, T> SubSyntaxTree<'rlt, T> {
    #[allow(private_bounds)]
    pub fn new(root: Uninit<'rlt, T>) -> Self
    where
        T: Into<AnyNode> + Identifiable,
    {
        let (root, mapping) = root.unwrap(NodeId::Root);
        Self {
            graph: Graph::new(root.into()),
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
        let id = self.graph.add_node_at_root(|id| {
            let node = node.unwrap(id.into());
            rlt = node.1;
            node.0.into()
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
        let (id, mapping) = self
            .graph
            .attach_subgraph_at(NodeKey::Root, subtree.graph)
            .unwrap();
        for &to in mapping.values() {
            self.graph[to].set_id(to.into());
        }
        if let Some(x) = self.graph.edge_weight_mut(id) {
            *x = TAG;
        }
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

    pub fn with_children_from<'a, const TAG: ChildTag, U>(
        mut self,
        iter: impl IntoIterator<Item = &'a (impl PopulateTree<Root = U> + 'a)>,
        context: &mut impl CodeHolder,
    ) -> Self
    where
        T: HasChildrenMarker<U, TAG>,
        'a: 'rlt
    {
        for node in iter {
            let subtree = node.convert(context);
            self.attach_subtree(subtree)
        }
        self
    }
    
    pub fn cast<R>(self) -> SubSyntaxTree<'rlt, R> where T: Into<R> {
        SubSyntaxTree {
            graph: self.graph,
            rlt_mapping: self.rlt_mapping,
            root_rlt_mapping: self.root_rlt_mapping,
            _phantom: PhantomData,
        }
    }
}
