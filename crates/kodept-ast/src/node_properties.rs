use crate::graph::{Change, GenericASTNode, GhostToken, Identifiable, SyntaxTree};
use kodept_core::{ConvertibleToMut, ConvertibleToRef};

pub trait NodeWithParent {
    type Parent;
}

#[allow(private_bounds)]
pub trait Node: Identifiable + Into<GenericASTNode> {
    fn parent<'b>(&self, tree: &'b SyntaxTree, token: &'b GhostToken) -> &'b Self::Parent
    where
        Self: NodeWithParent,
        GenericASTNode: ConvertibleToRef<Self::Parent>,
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
        token: &'b mut GhostToken,
    ) -> &'b mut Self::Parent
    where
        Self: NodeWithParent,
        GenericASTNode: ConvertibleToMut<Self::Parent>,
    {
        let id = self.get_id();
        tree.parent_of_mut(id, token)
    }

    fn replace_with(&self, other: Self) -> Change {
        Change::Replace {
            from_id: self.get_id().cast(),
            to: other.into(),
        }
    }

    fn remove(&self) -> Change {
        Change::DeleteSelf {
            node_id: self.get_id().cast(),
        }
    }
}
