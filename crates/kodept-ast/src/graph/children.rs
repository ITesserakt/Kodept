use crate::graph::generic_node::GenericASTNode;
use crate::graph::traits::Identifiable;
use crate::graph::utils::FromOptVec;
use crate::graph::SyntaxTree;

pub(crate) trait HasChildrenMarker<Child>: Identifiable {
    type Container: FromOptVec<T = Child>;

    fn get_children<'b>(&self, tree: &'b SyntaxTree) -> ChildrenRef<'b, Self, Child>
    where
        for<'a> &'a Child: TryFrom<&'a GenericASTNode>,
        Self: 'static,
    {
        Self::Container::unwrap(tree.children_of(self.get_id()))
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
