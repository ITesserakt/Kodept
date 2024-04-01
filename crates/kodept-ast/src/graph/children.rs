use kodept_core::{ConvertibleToMut, ConvertibleToRef};

use crate::graph::tags::ChildTag;
use crate::graph::utils::FromOptVec;
use crate::graph::{GenericASTNode, Identifiable};
use crate::graph::{GhostToken, SyntaxTree};

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

    fn get_children<'b>(
        &self,
        tree: &'b SyntaxTree,
        token: &'b GhostToken,
    ) -> ChildrenRef<'b, Self, Child, TAG>
    where
        GenericASTNode: ConvertibleToRef<Child>,
    {
        Self::Container::unwrap(tree.children_of(self.get_id(), token, TAG))
    }

    fn get_children_mut<'b>(&self, tree: &'b SyntaxTree) -> ChildrenMut<'b, Self, Child, TAG>
    where
        GenericASTNode: ConvertibleToMut<Child>,
    {
        Self::Container::unwrap_mut(tree.children_of_raw(self.get_id(), TAG))
    }
}

pub type ChildrenRef<'a, T, Child, const TAG: ChildTag> =
    <<T as HasChildrenMarker<Child, TAG>>::Container as FromOptVec>::Ref<'a>;

pub type ChildrenMut<'a, T, Child, const TAG: ChildTag> =
    <<T as HasChildrenMarker<Child, TAG>>::Container as FromOptVec>::Mut<'a>;

pub type ContainerT<T> = <T as FromOptVec>::T;

pub mod macros {
    #[macro_export]
    macro_rules! with_children {
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty as $tag:tt,)*}) => {
            paste::paste! {
            $(
            impl $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>, $tag> for $t {
                type Container = $c_t;
            }

            impl $t {
                $vis fn $name<'a>(
                    &self,
                    tree: &'a $crate::graph::SyntaxTree,
                    token: &'a $crate::graph::GhostToken,
                ) -> $crate::graph::ChildrenRef<'a, $t, $crate::graph::ContainerT<$c_t>, $tag> {
                    <Self as $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>, $tag>>::get_children(self, tree, token)
                }

                $vis fn [<$name _mut>]<'a>(
                    &self, tree: &'a $crate::graph::SyntaxTree
                ) -> $crate::graph::ChildrenMut<'a, $t, $crate::graph::ContainerT<$c_t>, $tag> {
                    <Self as $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>, $tag>>::get_children_mut(self, tree)
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
}
