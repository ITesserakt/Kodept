use crate::{Data, Node, NodeFamily, Visit, Visitor};
use std::marker::PhantomData;

pub struct NodeVec<'v, F, N> {
    pub inner: &'v mut Vec<N>,
    _phantom: PhantomData<F>,
}

impl<'v, F, V, N> Node<V> for NodeVec<'v, F, N>
where
    V: Visitor<F> + Visit<Self>,
    F: NodeFamily<V>,
{
    type Family = F;
}

impl<'v, V, N> Visit<NodeVec<'v, N::Family, N>> for V
where
    N: Node<V>,
    V: Visit<N> + Visitor<N::Family>,
    V::Output: Default,
{
    fn visit(
        &mut self,
        node: &mut NodeVec<N::Family, N>,
        metadata: &Data<Self, NodeVec<N::Family, N>>,
    ) -> Self::Output {
        if let Some((last, body)) = node.inner.split_last_mut() {
            for node in body {
                node.accept(self, metadata);
            }
            return last.accept(self, metadata);
        }
        Self::Output::default()
    }
}

pub trait VecExtension<N> {
    fn accept_vec<'v, V>(
        &'v mut self,
        visitor: &mut V,
        metadata: &Data<V, N>,
    ) -> <V as Visitor<N::Family>>::Output
    where
        V: Visit<NodeVec<'v, N::Family, N>>,
        V: Visit<N>,
        V: Visitor<N::Family>,
        N: Node<V> + 'v;
}

impl<N> VecExtension<N> for Vec<N> {
    fn accept_vec<'v, V>(
        &'v mut self,
        visitor: &mut V,
        metadata: &Data<V, N>,
    ) -> <V as Visitor<N::Family>>::Output
    where
        V: Visit<NodeVec<'v, N::Family, N>>,
        V: Visit<N>,
        V: Visitor<N::Family>,
        N: Node<V> + 'v,
    {
        visitor.visit(
            &mut NodeVec {
                inner: self,
                _phantom: PhantomData,
            },
            metadata,
        )
    }
}
