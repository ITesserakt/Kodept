use crate::graph::tags::ChildTag;
use crate::graph::utils::OptVec;
use crate::graph::{GenericASTNode, GenericNodeId, HasChildrenMarker, NodeId};
use crate::Uninit;

pub type ChangeSet = OptVec<Change>;

/// Represents a modification of AST
pub enum Change {
    /// Child removed
    Delete {
        parent_id: GenericNodeId,
        child_id: GenericNodeId,
    },
    /// Child added
    Add {
        parent_id: GenericNodeId,
        child: Uninit<GenericASTNode>,
        tag: ChildTag,
    },
    /// Replace itself with other node
    Replace {
        from_id: GenericNodeId,
        to: Uninit<GenericASTNode>,
    },
    /// Delete itself from ast hierarchy
    DeleteSelf { node_id: GenericNodeId },
}

impl Change {
    pub fn delete<T: Into<GenericASTNode>>(id: NodeId<T>) -> Change {
        Change::DeleteSelf {
            node_id: id.widen(),
        }
    }

    pub fn add<T, U, const TAG: ChildTag>(to: NodeId<T>, element: Uninit<U>) -> Change
    where
        T: Into<GenericASTNode> + HasChildrenMarker<U, TAG>,
        U: Into<GenericASTNode>,
    {
        Change::Add {
            parent_id: to.widen(),
            child: element.map_into(),
            tag: TAG,
        }
    }

    pub fn replace<T: Into<GenericASTNode>>(node: NodeId<GenericASTNode>, with: Uninit<T>) -> Change {
        Change::Replace {
            from_id: node.widen(),
            to: with.map_into(),
        }
    }
}
