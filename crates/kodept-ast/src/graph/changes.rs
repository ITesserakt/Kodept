use crate::graph::{GenericASTNode, NodeId, SyntaxTree};

#[derive(Default)]
pub struct ChangeSet(Vec<Change>);

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
    },
    /// Replace itself with other node
    Replace {
        from_id: NodeId<GenericASTNode>,
        to: GenericASTNode,
    },
    /// Delete itself from ast hierarchy
    DeleteSelf { node_id: NodeId<GenericASTNode> },
}

impl ChangeSet {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn add(change: Change, tree: &SyntaxTree) {}

    pub fn merge(self, other: Self) -> Self {
        self
    }
}
