use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::Identifiable;
use crate::graph::utils::FromOptVec;
use crate::graph::SyntaxTree;
use std::fmt::Debug;

pub trait HasChildrenMarker<Child>: Identifiable {
    type Container: FromOptVec<T = Child>;

    fn get_children<'b>(&self, tree: &'b SyntaxTree) -> ChildrenRef<'b, Self, Child>
    where
        for<'a> &'a Child: TryFrom<&'a GenericASTNode>,
        Self: 'static,
        for<'a> <&'a GenericASTNode as TryInto<&'a Child>>::Error: Debug,
    {
        Self::Container::unwrap(tree.children_of(self.get_id()))
    }

    fn for_children_mut<'b, F, T>(&self, tree: &'b mut SyntaxTree, handler: F) -> Vec<T>
    where
        for<'a> &'a mut Child: TryFrom<&'a mut GenericASTNode>,
        Self: 'static,
        F: FnMut(&mut Child) -> T,
    {
        tree.children_of_id(self.get_id(), handler).into_vec()
    }
}

pub(crate) type ChildrenRef<'a, T, Child> =
    <<T as HasChildrenMarker<Child>>::Container as FromOptVec>::Ref<'a>;

pub(crate) type ChildrenMut<'a, T, Child> =
    <<T as HasChildrenMarker<Child>>::Container as FromOptVec>::Mut<'a>;

pub(crate) type ContainerT<T> = <T as FromOptVec>::T;

pub(crate) mod macros {
    #[macro_export]
    macro_rules! with_children {
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty)*}) => {
            paste::paste! {
            $(
            #[allow(private_interfaces)]
            impl $crate::graph::children::HasChildrenMarker<$crate::graph::children::ContainerT<$c_t>> for $t {
                type Container = $c_t;
            }

            #[allow(private_interfaces)]
            #[allow(clippy::needless_lifetimes)]
            impl $t {
                $vis fn $name<'a>(&self, tree: &'a $crate::graph::SyntaxTree) -> $crate::graph::children::ChildrenRef<'a, $t, $crate::graph::children::ContainerT<$c_t>> {
                    <Self as $crate::graph::children::HasChildrenMarker<$crate::graph::children::ContainerT<$c_t>>>::get_children(self, tree)
                }

                $vis fn [<$name _mut>]<'a>(&self, tree: &'a mut $crate::graph::SyntaxTree) {
                    <Self as $crate::graph::children::HasChildrenMarker<$crate::graph::children::ContainerT<$c_t>>>::for_children_mut(self, tree, |_| 1);
                }
            })*
            }
        };
    }
}
