use crate::graph::any_node::AnyNode;
use crate::graph::children::tags::ChildTag;
use crate::graph::{SyntaxTree};
use crate::traits::Identifiable;
use kodept_core::ConvertibleToRef;
use crate::graph::utils::{ContainerFamily, ContainerT};

pub mod tags {
    pub type ChildTag = u8;

    pub const NO_TAG: ChildTag = 0;
    pub const PRIMARY: ChildTag = 1;
    pub const SECONDARY: ChildTag = 2;
    pub const LEFT: ChildTag = 3;
    pub const RIGHT: ChildTag = 4;

    pub static TAGS_DESC: [&str; 5] = ["", "P", "S", "L", "R"];
}

pub trait HasChildrenMarker<Child, const TAG: ChildTag>: Identifiable {
    type Container: ContainerFamily;

    fn get_children<'b, P>(
        &self,
        tree: &'b SyntaxTree<P>,
    ) -> ContainerT<Self::Container, &'b Child>
    where
        AnyNode: ConvertibleToRef<Child>,
    {
        Self::Container::from_iter(tree.children_of(self.get_id(), TAG))
    }
}

pub(crate) mod macros {
    macro_rules! with_children {
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty as $tag:tt,)*}) => {
            $(
            impl $crate::graph::HasChildrenMarker<$crate::graph::InnerT<$c_t>, $tag> for $t {
                type Container = $crate::graph::FamilyT<$c_t>;
            }

            impl $t {
                #[inline(always)]
                $vis fn $name<'a>(
                    &self,
                    tree: &'a $crate::graph::SyntaxTree
                ) -> $crate::graph::WrapRef<'a, $c_t> {
                    <Self as $crate::graph::HasChildrenMarker<$crate::graph::InnerT<$c_t>, $tag>>::get_children(self, tree)
                }
            })*
        };
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty,)*}) => {
            $crate::with_children! { $t => {
                $($vis $name: $c_t as 0,)*
            } }
        }
    }

    pub(crate) use with_children;
}
