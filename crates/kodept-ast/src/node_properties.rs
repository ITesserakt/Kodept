use kodept_core::{ConvertibleToMut, ConvertibleToRef};

use crate::graph::{Change, AnyNode, PermTkn, Identifiable, SyntaxTree, NodeId};

#[repr(transparent)]
pub struct Uninit<T>(T);

impl<T> Uninit<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    #[allow(private_bounds)]
    pub fn unwrap(self, id: NodeId<T>) -> T
        where
            T: crate::graph::Identifiable,
    {
        self.0.set_id(id);
        self.0
    }

    #[inline]
    pub fn map_into<U>(self) -> Uninit<U>
        where
            T: Into<U>,
    {
        Uninit(self.0.into())
    }
}

pub trait NodeWithParent {
    type Parent;
}

#[allow(private_bounds)]
pub trait Node: Identifiable + Into<AnyNode> {
    fn parent<'b>(&self, tree: &'b SyntaxTree, token: &'b PermTkn) -> &'b Self::Parent
    where
        Self: NodeWithParent,
        AnyNode: ConvertibleToRef<Self::Parent>,
    {
        let id = self.get_id();
        tree.parent_of(id, token)
            .expect("NodeWithParent: contract violated")
            .try_as_ref()
            .expect("Node has wrong type")
    }

    fn parent_mut<'b>(
        &self,
        tree: &'b mut SyntaxTree,
        token: &'b mut PermTkn,
    ) -> &'b mut Self::Parent
    where
        Self: NodeWithParent,
        AnyNode: ConvertibleToMut<Self::Parent>,
    {
        let id = self.get_id();
        tree.parent_of_mut(id, token)
    }

    fn replace_with(&self, other: Uninit<Self>) -> Change {
        Change::Replace {
            from_id: self.get_id().widen(),
            to: other.map_into(),
        }
    }

    fn remove(&self) -> Change {
        Change::DeleteSelf {
            node_id: self.get_id().widen(),
        }
    }
}
