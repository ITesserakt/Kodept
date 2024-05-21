use kodept_core::ConvertibleTo;

use crate::graph::{AnyNode, NodeId};

#[repr(transparent)]
pub struct Uninit<T>(T);

impl<T> Uninit<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    #[allow(private_bounds)]
    pub fn unwrap(self, id: NodeId<T>) -> T
    where
        T: crate::graph::Identifiable,
    {
        self.0.set_id(id);
        self.0
    }

    #[inline]
    pub fn map_into<U>(self) -> Uninit<U>
    where
        T: Into<U>,
    {
        Uninit(self.0.into())
    }
}

pub trait ForceInto {
    type Output<T: 'static>;

    fn force_into<T>(self) -> Self::Output<T>
    where
        T: 'static,
        Self: ConvertibleTo<Self::Output<T>>;
}

impl<'a> ForceInto for &'a AnyNode {
    type Output<T: 'static> = &'a T;

    #[inline]
    fn force_into<T: 'static>(self) -> Self::Output<T>
    where
        Self: ConvertibleTo<Self::Output<T>>,
    {
        self.try_as().expect("Cannot convert $crate::graph::AnyNode")
    }
}

impl<'a> ForceInto for &'a mut AnyNode {
    type Output<T: 'static> = &'a mut T;

    #[inline]
    fn force_into<T>(self) -> Self::Output<T>
    where
        T: 'static,
        Self: ConvertibleTo<Self::Output<T>>,
    {
        self.try_as().expect("Cannot convert $crate::graph::AnyNode")
    }
}

pub mod implementation {
    #[macro_export]
    macro_rules! wrapper {
    ($(#[$config:meta])* $vis:vis wrapper $wrapper:ident {
        $($name:ident($t:ty) = $variants:pat $(if $variant_if:expr)? => $variant_expr:expr$(,)*)*
    }) => {paste::paste! {
        $(#[$config])*
        #[repr(transparent)]
        $vis struct $wrapper($crate::graph::AnyNode);

        #[derive(derive_more::From)]
        $vis enum [<$wrapper Enum>]<'lt> {
            $([<$name:camel>](&'lt $t),)*
        }

        #[derive(derive_more::From)]
        $vis enum [<$wrapper EnumMut>]<'lt> {
            $([<$name:camel>](&'lt mut $t),)*
        }

        #[allow(unsafe_code)]
        unsafe impl $crate::graph::NodeUnion for $wrapper {
            fn contains(node: &$crate::graph::AnyNode) -> bool {
                #[allow(unused_variables)]
                #[allow(unreachable_patterns)]
                match node {
                    $($variants $(if $variant_if)? => true,)*
                    _ => false
                }
            }
        }

        impl<'a> TryFrom<&'a $crate::graph::AnyNode> for &'a $wrapper {
            type Error = $crate::utils::Skip<<&'a $crate::graph::AnyNode as TryFrom<&'a $crate::graph::AnyNode>>::Error>;

            #[inline]
            fn try_from(value: &'a $crate::graph::AnyNode) -> Result<Self, Self::Error> {
                if !<$wrapper as $crate::graph::NodeUnion>::contains(value) {
                    return Err($crate::utils::Skip::Skipped);
                }
                Ok(<$wrapper as $crate::graph::NodeUnion>::wrap(value))
            }
        }

        impl<'a> TryFrom<&'a mut $crate::graph::AnyNode> for &'a mut $wrapper {
            type Error = $crate::utils::Skip<<&'a mut $crate::graph::AnyNode as TryFrom<&'a mut $crate::graph::AnyNode>>::Error>;

            #[inline]
            fn try_from(value: &'a mut $crate::graph::AnyNode) -> Result<Self, Self::Error> {
                if !<$wrapper as $crate::graph::NodeUnion>::contains(value) {
                    return Err($crate::utils::Skip::Skipped);
                }
                Ok(<$wrapper as $crate::graph::NodeUnion>::wrap_mut(value))
            }
        }

        $(
        impl From<$t> for $wrapper {
            #[inline]
            fn from(value: $t) -> Self {
                let generic: $crate::graph::AnyNode = value.into();
                $wrapper(generic)
            }
        }
        )*

        impl $crate::traits::Identifiable for $wrapper {
            fn get_id(&self) -> $crate::graph::NodeId<Self> {
                <$crate::graph::AnyNode as $crate::traits::Identifiable>::get_id(&self.0).narrow()
            }
        }

        impl $wrapper {
            pub fn as_enum(&self) -> [<$wrapper Enum>] {
                match self {
                    $($wrapper($variants) $(if $variant_if)? => $variant_expr,)*
                    _ => unreachable!()
                }
            }

            pub fn as_enum_mut(&mut self) -> [<$wrapper EnumMut>] {
                match self {
                    $($wrapper($variants) $(if $variant_if)? => $variant_expr,)*
                    _ => unreachable!()
                }
            }
        }
    }}
}
    
    macro_rules! functor_map {
        ($ty:ident, $self:expr, |$var:ident| $mapping:expr) => {
            match $self {
                $ty::File($var) => $mapping,
                $ty::Module($var) => $mapping,
                $ty::Struct($var) => $mapping,
                $ty::Enum($var) => $mapping,
                $ty::TypedParameter($var) => $mapping,
                $ty::UntypedParameter($var) => $mapping,
                $ty::TypeName($var) => $mapping,
                $ty::Variable($var) => $mapping,
                $ty::InitializedVariable($var) => $mapping,
                $ty::BodiedFunction($var) => $mapping,
                $ty::ExpressionBlock($var) => $mapping,
                $ty::Application($var) => $mapping,
                $ty::Lambda($var) => $mapping,
                $ty::Reference($var) => $mapping,
                $ty::Access($var) => $mapping,
                $ty::Number($var) => $mapping,
                $ty::Char($var) => $mapping,
                $ty::String($var) => $mapping,
                $ty::Tuple($var) => $mapping,
                $ty::If($var) => $mapping,
                $ty::Elif($var) => $mapping,
                $ty::Else($var) => $mapping,
                $ty::Binary($var) => $mapping,
                $ty::Unary($var) => $mapping,
                $ty::AbstractFunction($var) => $mapping,
                $ty::ProdType($var) => $mapping,
            }
        };
    }

    macro_rules! node {
    ($(#[$config:meta])* $vis:vis struct $name:ident {
        $($field_vis:vis $field_name:ident: $field_type:ty,)*;
        $($graph_vis:vis $graph_name:ident: $graph_type:ty$( as $tag:tt)?,)*
    }) => {
        #[cfg(feature = "serde")]
        $(#[$config])*
        $vis struct $name {
            id: once_cell_serde::unsync::OnceCell<$crate::graph::NodeId<$name>>,
            $($field_vis $field_name: $field_type,)*
        }

        #[cfg(not(feature = "serde"))]
        $(#[$config])*
        $vis struct $name {
            id: std::cell::OnceCell<$crate::graph::NodeId<$name>>,
            $($field_vis $field_name: $field_type,)*
        }

        impl $crate::node_properties::Node for $name {}

        impl $name {
            pub fn uninit($($field_name: $field_type,)*) -> $crate::Uninit<Self> {
                $crate::Uninit::new(Self {
                    id: Default::default(),
                    $(
                    $field_name,
                    )*
                })
            }

            pub fn into_uninit(self) -> $crate::Uninit<Self> {
                Self::uninit($(self.$field_name, )*)
            }
        }

        impl $crate::graph::Identifiable for $name {
            fn get_id(&self) -> $crate::graph::NodeId<Self> {
                self.id.get().copied().expect("Unreachable")
            }

            fn set_id(&self, value: $crate::graph::NodeId<Self>) {
                if let Err(_) = self.id.set(value) {
                    tracing::warn!("Tried to set id twice");
                }
            }
        }

        $crate::with_children! [$name => {
            $($graph_vis $graph_name: $graph_type $( as $tag)?,)*
        }];
    };
    ($(#[$config:meta])* $vis:vis struct $name:ident;) => {
        node!($(#[$config])* $vis struct $name {;});
    }
}

    pub(crate) use {functor_map, node};
}
