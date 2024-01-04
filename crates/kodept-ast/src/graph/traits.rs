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

pub(crate) mod macros {
    #[macro_export]
    macro_rules! impl_identifiable {
        ($($t:ty$(,)?)*) => {
            $($crate::property!(in mut $crate::graph::traits::Identifiable => $t, id: NodeId<Self>);)*
        };
    }

    #[macro_export]
    macro_rules! node {
        ($(#[$config:meta])* $vis:vis struct $name:ident {
            $($field_vis:vis $field_name:ident: $field_type:ty,)*;
            $($graph_vis:vis $graph_name:ident: $graph_type:ty,)*
        }) => {
            $(#[$config])*
            $vis struct $name {
                id: $crate::node_id::NodeId<$name>,
                $($field_vis $field_name: $field_type,)*
            }

            $crate::impl_identifiable!($name);

            $crate::with_children! [$name => {
                $($graph_vis $graph_name: $graph_type)*
            }];
        };
        ($(#[$config:meta])* $vis:vis struct $name:ident;) => {
            node!($(#[$config])* $vis struct $name {;});
        }
    }
}
