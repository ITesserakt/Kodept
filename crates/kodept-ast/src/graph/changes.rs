use crate::graph::{GenericASTNode, NodeId};
use crate::graph::tags::ChildTag;
use crate::graph::utils::OptVec;

pub type ChangeSet = OptVec<Change>;

/// Represents a modification of AST
pub enum Change {
    /// Child removed
    Delete {
        parent_id: NodeId<GenericASTNode>,
        child_id: NodeId<GenericASTNode>,
    },
    /// Child added
    Add {
        parent_id: NodeId<GenericASTNode>,
        child: GenericASTNode,
        tag: ChildTag,
    },
    /// Replace itself with other node
    Replace {
        from_id: NodeId<GenericASTNode>,
        to: GenericASTNode,
    },
    /// Delete itself from ast hierarchy
    DeleteSelf { node_id: NodeId<GenericASTNode> },
}

impl Change {
    pub fn delete<T: Into<GenericASTNode>>(id: NodeId<T>) -> Change {
        Change::DeleteSelf { node_id: id.cast() }
    }

    pub fn add<T, U>(to: NodeId<T>, element: U, tag: ChildTag) -> Change
    where
        T: Into<GenericASTNode>,
        U: Into<GenericASTNode>,
    {
        Change::Add {
            parent_id: to.cast(),
            child: element.into(),
            tag,
        }
    }

    pub fn replace<T: Into<GenericASTNode>>(node: NodeId<GenericASTNode>, with: T) -> Change {
        Change::Replace {
            from_id: node.cast(),
            to: with.into(),
        }
    }
}
