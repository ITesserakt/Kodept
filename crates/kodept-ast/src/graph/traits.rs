use crate::graph::node_id::NodeId;

pub(crate) trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
    fn set_id(&self, value: NodeId<Self>);
}
