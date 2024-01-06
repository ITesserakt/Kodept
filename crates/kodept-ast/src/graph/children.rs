use std::fmt::Debug;

use crate::graph::utils::FromOptVec;
use crate::graph::{GenericASTNode, Identifiable};
use crate::graph::{GhostToken, SyntaxTree};

pub trait HasChildrenMarker<Child>: Identifiable {
    type Container: FromOptVec<T = Child>;

    fn get_children<'b>(
        &self,
        tree: &'b SyntaxTree,
        token: &'b GhostToken,
    ) -> ChildrenRef<'b, Self, Child>
    where
        for<'a> &'a Child: TryFrom<&'a GenericASTNode>,
        for<'a> <&'a GenericASTNode as TryInto<&'a Child>>::Error: Debug,
    {
        Self::Container::unwrap(tree.children_of(self.get_id(), token))
    }
}

pub trait HasChildrenMutMarker<Child>: Identifiable {
    type Container: FromOptVec<T = Child>;

    fn get_children_mut<'b>(&self, tree: &'b SyntaxTree) -> ChildrenMut<'b, Self, Child>
    where
        for<'a> &'a mut Child: TryFrom<&'a mut GenericASTNode>,
    {
        Self::Container::unwrap_mut(tree.children_of_raw(self.get_id()))
    }
}

pub type ChildrenRef<'a, T, Child> =
    <<T as HasChildrenMarker<Child>>::Container as FromOptVec>::Ref<'a>;

pub type ChildrenMut<'a, T, Child> =
    <<T as HasChildrenMutMarker<Child>>::Container as FromOptVec>::Mut<'a>;

pub type ContainerT<T> = <T as FromOptVec>::T;

pub mod macros {
    #[macro_export]
    macro_rules! with_children {
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty)*}) => {
            paste::paste! {
            $(
            impl $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>> for $t {
                type Container = $c_t;
            }
            impl $crate::graph::HasChildrenMutMarker<$crate::graph::ContainerT<$c_t>> for $t {
                type Container = $c_t;
            }

            impl $t {
                $vis fn $name<'a>(
                    &self,
                    tree: &'a $crate::graph::SyntaxTree,
                    token: &'a $crate::graph::GhostToken,
                ) -> $crate::graph::ChildrenRef<'a, $t, $crate::graph::ContainerT<$c_t>> {
                    <Self as $crate::graph::HasChildrenMarker<$crate::graph::ContainerT<$c_t>>>::get_children(self, tree, token)
                }

                $vis fn [<$name _mut>]<'a>(
                    &self, tree: &'a $crate::graph::SyntaxTree
                ) -> $crate::graph::ChildrenMut<'a, $t, $crate::graph::ContainerT<$c_t>> {
                    <Self as $crate::graph::HasChildrenMutMarker<$crate::graph::ContainerT<$c_t>>>::get_children_mut(self, tree)
                }
            })*
            }
        };
    }
}
