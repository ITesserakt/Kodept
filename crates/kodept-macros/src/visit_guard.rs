use derive_more::Constructor;
use kodept_ast::graph::NodeId;
use kodept_ast::visit_side::VisitSide;

#[derive(Debug, Constructor)]
pub struct VisitGuard<T> {
    node: NodeId<T>,
    side: VisitSide,
}

impl<T> VisitGuard<T> {
    pub fn allow_all(self) -> (NodeId<T>, VisitSide) {
        (self.node, self.side)
    }

    pub fn allow_only(self, side: VisitSide) -> Option<NodeId<T>> {
        (self.side == side).then_some(self.node)
    }
    
    pub fn allow_last(self) -> Option<NodeId<T>> {
        matches!(self.side, VisitSide::Exiting | VisitSide::Leaf).then_some(self.node)
    }
}
