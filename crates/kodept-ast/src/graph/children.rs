use crate::graph::any_node::AnyNode;
use crate::graph::children::tags::ChildTag;
use crate::graph::nodes::PermTkn;
use crate::graph::utils::FromOptVec;
use crate::graph::SyntaxTree;
use crate::traits::Identifiable;
use kodept_core::{ConvertibleToMut, ConvertibleToRef};

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
        token: &'b PermTkn,
    ) -> ChildrenRef<'b, Self, Child, TAG>
    where
        AnyNode: ConvertibleToRef<Child>,
    {
        Self::Container::unwrap(tree.children_of(self.get_id(), token, TAG))
    }

    fn get_children_mut<'b>(&self, tree: &'b SyntaxTree) -> ChildrenMut<'b, Self, Child, TAG>
    where
        AnyNode: ConvertibleToMut<Child>,
    {
        Self::Container::unwrap_mut(tree.raw_children_of(self.get_id(), TAG))
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
                $vis fn $name<'a>(
                    &self,
                    tree: &'a $crate::graph::SyntaxTree,
                    token: &'a $crate::graph::PermTkn,
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
    
    pub(crate) use with_children;
}
