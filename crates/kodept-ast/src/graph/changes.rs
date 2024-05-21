use crate::graph::tags::ChildTag;
use crate::graph::utils::OptVec;
use crate::graph::{AnyNode, GenericNodeId, HasChildrenMarker, NodeId};
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
        child: Uninit<AnyNode>,
        tag: ChildTag,
    },
    /// Replace itself with other node
    Replace {
        from_id: GenericNodeId,
        to: Uninit<AnyNode>,
    },
    /// Delete itself from ast hierarchy
    DeleteSelf { node_id: GenericNodeId },
}

impl Change {
    pub fn delete<T: Into<AnyNode>>(id: NodeId<T>) -> Change {
        Change::DeleteSelf {
            node_id: id.widen(),
        }
    }

    pub fn add<T, U, const TAG: ChildTag>(to: NodeId<T>, element: Uninit<U>) -> Change
    where
        T: Into<AnyNode> + HasChildrenMarker<U, TAG>,
        U: Into<AnyNode>,
    {
        Change::Add {
            parent_id: to.widen(),
            child: element.map_into(),
            tag: TAG,
        }
    }

    pub fn replace<T: Into<AnyNode>>(node: NodeId<AnyNode>, with: Uninit<T>) -> Change {
        Change::Replace {
            from_id: node.widen(),
            to: with.map_into(),
        }
    }
}
