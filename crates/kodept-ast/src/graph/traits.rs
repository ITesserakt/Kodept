use crate::graph::NodeId;

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
    fn set_id(&mut self, value: NodeId<Self>);
}
