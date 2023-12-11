#![allow(clippy::needless_lifetimes)]

use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::Identifiable;
use crate::graph::SyntaxTree;
use smallvec::SmallVec;
use std::any::type_name;

pub(crate) trait HasChildrenMarker<Child>: Identifiable {
    type Container: Container<T = Child>;

    fn get_children<'b>(&self, tree: &'b SyntaxTree) -> ChildrenRef<'b, Self, Child>
    where
        for<'a> &'a Child: TryFrom<&'a GenericASTNode>,
        Self: 'static,
    {
        Self::Container::unwrap(tree.children_of(self.get_id()))
    }
}

#[repr(transparent)]
pub struct Identity<T>(pub T);

pub(crate) type ChildrenRef<'a, T, Child> =
    <<T as HasChildrenMarker<Child>>::Container as Container>::Ref<'a>;

pub(crate) type ChildrenMut<'a, T, Child> =
    <<T as HasChildrenMarker<Child>>::Container as Container>::Mut<'a>;

pub(crate) type ContainerT<T> = <T as Container>::T;

pub(crate) trait Container {
    type Ref<'a>
    where
        Self::T: 'a;
    type Mut<'a>
    where
        Self::T: 'a;
    type T;

    fn unwrap<'a>(value: SmallVec<&'a Self::T, 1>) -> Self::Ref<'a>;
    fn unwrap_mut<'a>(value: SmallVec<&'a mut Self::T, 1>) -> Self::Mut<'a>;
}

impl<T> Container for Option<T> {
    type Ref<'a>= Option<&'a T> where T: 'a;
    type Mut<'a> = Option<&'a mut T> where T: 'a;
    type T = T;

    fn unwrap<'a>(value: SmallVec<&'a Self::T, 1>) -> Self::Ref<'a> {
        match value.as_slice() {
            [x] => Some(x),
            [] => None,
            _ => panic!(
                "Container must has no more then one child <{}>, but has {}",
                type_name::<T>(),
                value.len()
            ),
        }
    }

    fn unwrap_mut<'a>(mut value: SmallVec<&'a mut Self::T, 1>) -> Self::Mut<'a> {
        let len = value.len();
        match value.len() {
            1 => value.pop(),
            0 => None,
            _ => panic!(
                "Container must has no more then one child <{}>, but has {}",
                type_name::<T>(),
                len
            ),
        }
    }
}

impl<T> Container for Vec<T> {
    type Ref<'a> = Vec<&'a T> where Self::T: 'a;
    type Mut<'a> = Vec<&'a mut T> where Self::T: 'a;
    type T = T;

    fn unwrap<'a>(value: SmallVec<&'a Self::T, 1>) -> Self::Ref<'a> {
        value.into_vec()
    }

    fn unwrap_mut<'a>(value: SmallVec<&'a mut Self::T, 1>) -> Self::Mut<'a> {
        value.into_vec()
    }
}

impl<T> Container for Identity<T> {
    type Ref<'a> = &'a T where Self::T: 'a;
    type Mut<'a> = &'a mut T where Self::T: 'a;
    type T = T;

    fn unwrap<'a>(value: SmallVec<&'a Self::T, 1>) -> Self::Ref<'a> {
        match value.as_slice() {
            [x] => x,
            _ => panic!(
                "Container must has only one child <{}>, but has {}",
                type_name::<T>(),
                value.len()
            ),
        }
    }

    fn unwrap_mut<'a>(mut value: SmallVec<&'a mut Self::T, 1>) -> Self::Mut<'a> {
        match value.len() {
            1 => value.pop().unwrap(),
            _ => panic!(
                "Container must has only one child <{}>, but has {}",
                type_name::<T>(),
                value.len()
            ),
        }
    }
}

pub(crate) mod macros {
    #[macro_export]
    macro_rules! with_children {
        ($t:ty => {$($vis:vis $name:ident: $c_t:ty)*}) => {
            paste::paste! {
            #[allow(private_interfaces)]
            $(impl $crate::graph::children::HasChildrenMarker<$crate::graph::children::ContainerT<$c_t>> for $t {
                type Container = $c_t;
            }

            impl $t {
                #[allow(private_interfaces)]
                $vis fn $name<'a>(&self, tree: &'a $crate::graph::SyntaxTree) -> $crate::graph::children::ChildrenRef<'a, $t, $crate::graph::children::ContainerT<$c_t>> {
                    <Self as $crate::graph::children::HasChildrenMarker<$crate::graph::children::ContainerT<$c_t>>>::get_children(self, tree)
                }
            })*
            }
        };
    }
}
