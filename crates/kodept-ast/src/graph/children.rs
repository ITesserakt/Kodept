use crate::graph::any_node::AnyNode;
use crate::graph::children::tags::ChildTag;
use crate::graph::utils::FromOptVec;
use crate::graph::{CanAccess, SyntaxTree};
use crate::traits::Identifiable;
use kodept_core::ConvertibleToRef;

pub mod tags {
    pub type ChildTag = u8;

    pub const PRIMARY: ChildTag = 1;
    pub const SECONDARY: ChildTag = 2;
    pub const LEFT: ChildTag = 3;
    pub const RIGHT: ChildTag = 4;

    pub static TAGS_DESC: [&str; 5] = ["", "P", "S", "L", "R"];
}

pub trait HasChildrenMarker<Child, const TAG: ChildTag>: Identifiable {
    type Container: FromOptVec<T = Child>;

    fn get_children<'b, P: CanAccess>(
        &self,
        tree: &'b SyntaxTree<P>,
    ) -> ChildrenRef<'b, Self, Child, TAG>
    where
        AnyNode: ConvertibleToRef<Child>,
    {
        Self::Container::unwrap(tree.children_of(self.get_id(), TAG))
    }
}

pub(crate) type ChildrenRef<'a, T, Child, const TAG: ChildTag> =
    <<T as HasChildrenMarker<Child, TAG>>::Container as FromOptVec>::Ref<'a>;

pub(crate) type ChildrenMut<'a, T, Child, const TAG: ChildTag> =
    <<T as HasChildrenMarker<Child, TAG>>::Container as FromOptVec>::Mut<'a>;

pub(crate) type ContainerT<T> = <T as FromOptVec>::T;

pub(crate) mod macros {
    macro_rules! with_children {
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty as $tag:tt,)*}) => {
            paste::paste! {
            $(
            impl $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>, $tag> for $t {
                type Container = $c_t;
            }

            impl $t {
                #[inline(always)]
                $vis fn $name<'a, P: $crate::graph::CanAccess>(
                    &self,
                    tree: &'a $crate::graph::SyntaxTree<P>
                ) -> $crate::graph::ChildrenRef<'a, $t, $crate::graph::ContainerT<$c_t>, $tag> {
                    <Self as $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>, $tag>>::get_children(self, tree)
                }
            })*
            }
        };
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty,)*}) => {
            $crate::with_children! { $t => {
                $($vis $name: $c_t as 0,)*
            } }
        }
    }

    pub(crate) use with_children;
}
