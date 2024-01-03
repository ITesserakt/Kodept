use crate::graph::generic_node::GenericASTNode;
use crate::graph::SyntaxTree;
use crate::node_id::NodeId;
use crate::traits::Linker;
use kodept_core::structure::span::CodeHolder;

pub trait PopulateTree {
    type Output: Into<GenericASTNode> + ?Sized;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<<Self as PopulateTree>::Output>;
}

pub(crate) trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
    fn set_id(&mut self, value: NodeId<Self>);
}

#[macro_export]
macro_rules! impl_identifiable_2 {
    ($($t:ty$(,)*)*) => {
        $($crate::property!(in mut $crate::graph::traits::Identifiable => $t, id: NodeId<Self>);)*
    };
}
