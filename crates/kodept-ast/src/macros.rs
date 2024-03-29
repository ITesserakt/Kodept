#[macro_export]
macro_rules! wrapper {
    ($(#[$config:meta])* $vis:vis wrapper $wrapper:ident($inner:ty);) => {
        $(#[$config])*
        #[repr(transparent)]
        pub struct $wrapper($inner);

        impl<'a> TryFrom<&'a GenericASTNode> for &'a $wrapper {
            type Error = $crate::utils::Skip<<&'a $inner as TryFrom<&'a GenericASTNode>>::Error>;

            #[inline]
            fn try_from(value: &'a GenericASTNode) -> Result<Self, Self::Error> {
                let node: &$inner = value.try_into()?;
                Ok(unsafe { std::mem::transmute(node) })
            }
        }

        impl<'a> TryFrom<&'a mut GenericASTNode> for &'a mut $wrapper {
            type Error = $crate::utils::Skip<<&'a mut $inner as TryFrom<&'a mut GenericASTNode>>::Error>;

            #[inline]
            fn try_from(value: &'a mut GenericASTNode) -> Result<Self, Self::Error> {
                let node: &mut $inner = value.try_into()?;
                Ok(unsafe { std::mem::transmute(node) })
            }
        }
    };
    ($(#[$config:meta])* $vis:vis wrapper $wrapper:ident {
        $($name:ident($t:ty) = $variants:pat $(if $variant_if:expr)? => $variant_expr:expr$(,)*)*
    }) => {
        $(#[$config])*
        #[repr(transparent)]
        pub struct $wrapper(GenericASTNode);

        impl<'a> TryFrom<&'a GenericASTNode> for &'a $wrapper {
            type Error = $crate::utils::Skip<<&'a GenericASTNode as TryFrom<&'a GenericASTNode>>::Error>;

            #[inline]
            fn try_from(value: &'a GenericASTNode) -> Result<Self, Self::Error> {
                if !<$wrapper as $crate::graph::NodeUnion>::contains(value) {
                    return Err($crate::utils::Skip::Skipped);
                }
                Ok(unsafe { std::mem::transmute(value) })
            }
        }

        impl<'a> TryFrom<&'a mut GenericASTNode> for &'a mut $wrapper {
            type Error = $crate::utils::Skip<<&'a mut GenericASTNode as TryFrom<&'a mut GenericASTNode>>::Error>;

            #[inline]
            fn try_from(value: &'a mut GenericASTNode) -> Result<Self, Self::Error> {
                if !<$wrapper as $crate::graph::NodeUnion>::contains(value) {
                    return Err($crate::utils::Skip::Skipped);
                }
                Ok(unsafe { std::mem::transmute(value) })
            }
        }

        unsafe impl $crate::graph::NodeUnion for $wrapper {
            fn contains(node: &GenericASTNode) -> bool {
                #[allow(unused_variables)]
                #[allow(unreachable_patterns)]
                match node {
                    $($variants $(if $variant_if)? => true,)*
                    _ => false
                }
            }
        }

        $(
        impl From<$t> for $wrapper {
            #[inline]
            fn from(value: $t) -> Self {
                let generic: GenericASTNode = value.into();
                $wrapper(generic)
            }
        }
        )*

        impl $wrapper {
            paste::paste! {
                $(
                #[inline]
                pub fn [<as_ $name>](&self) -> Option<&$t> {
                    match self {
                        $wrapper($variants) $(if $variant_if)? => $variant_expr,
                        _ => None,
                    }
                }
                #[inline]
                pub fn [<as_ $name _mut>](&mut self) -> Option<&mut $t> {
                    match self {
                        $wrapper($variants) $(if $variant_if)? => $variant_expr,
                        _ => None
                    }
                }
                )*
            }
        }
    }
}

#[macro_export]
macro_rules! make_ast_node_adaptor {
    ($name:ident, lifetimes: [$($life:lifetime$(,)*)*], $wrapper:ident, configs: [$($cfg:meta$(,)*)*]) => {
        $(#[$cfg])*
        pub enum $name<$($life, )*> {
            File($wrapper<$($life, )* FileDeclaration>),
            Module($wrapper<$($life, )* ModuleDeclaration>),
            Struct($wrapper<$($life, )* StructDeclaration>),
            Enum($wrapper<$($life, )* EnumDeclaration>),
            TypedParameter($wrapper<$($life, )* TypedParameter>),
            UntypedParameter($wrapper<$($life, )* UntypedParameter>),
            TypeName($wrapper<$($life, )* TypeName>),
            Variable($wrapper<$($life, )* Variable>),
            InitializedVariable($wrapper<$($life, )* InitializedVariable>),
            BodiedFunction($wrapper<$($life, )* BodiedFunctionDeclaration>),
            ExpressionBlock($wrapper<$($life, )* ExpressionBlock>),
            Application($wrapper<$($life, )* Application>),
            Lambda($wrapper<$($life, )* Lambda>),
            Reference($wrapper<$($life, )* Reference>),
            Access($wrapper<$($life, )* Access>),
            Number($wrapper<$($life, )* NumberLiteral>),
            Char($wrapper<$($life, )* CharLiteral>),
            String($wrapper<$($life, )* StringLiteral>),
            Tuple($wrapper<$($life, )* TupleLiteral>),
            If($wrapper<$($life, )* IfExpression>),
            Elif($wrapper<$($life, )* ElifExpression>),
            Else($wrapper<$($life, )* ElseExpression>),
            Binary($wrapper<$($life, )* Binary>),
            Unary($wrapper<$($life, )* Unary>),
            AbstractFunction($wrapper<$($life, )* AbstractFunctionDeclaration>),
            ProdType($wrapper<$($life, )* ProdType>),
            SumType($wrapper<$($life, )* SumType>),
        }
    };
}

#[macro_export]
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
            $ty::SumType($var) => $mapping,
        }
    };
}

#[macro_export]
macro_rules! property {
    (in mut $trait_name:ty => $self_name:ty, $prop:ident: $prop_ty:ty) => {
        paste::paste! {
        impl $trait_name for $self_name {
            fn [<get_ $prop>](&self) -> $prop_ty {
                self.$prop
            }

            fn [<set_ $prop>](&mut self, value: $prop_ty) {
                self.$prop = value;
            }
        }
        }
    };
    (in $trait_name:ty => $self_name:ty, $prop:ident: $prop_ty:ty) => {
        paste::paste! {
        impl $trait_name for $self_name {
            fn [<get_ $prop>](&self) -> $prop_ty {
                self.$prop
            }
        }
        }
    };
}

#[macro_export]
macro_rules! impl_identifiable {
    ($($t:ty$(,)?)*) => {
        $($crate::property!(in mut $crate::graph::Identifiable => $t, id: NodeId<Self>);)*
    };
}

#[macro_export]
macro_rules! node {
    ($(#[$config:meta])* $vis:vis struct $name:ident {
        $($field_vis:vis $field_name:ident: $field_type:ty,)*;
        $($graph_vis:vis $graph_name:ident: $graph_type:ty$( as $tag:tt)*,)*
    }) => {
        $(#[$config])*
        $vis struct $name {
            id: $crate::graph::NodeId<$name>,
            $($field_vis $field_name: $field_type,)*
        }

        impl $crate::graph::Node for $name {}

        $crate::impl_identifiable!($name);

        $crate::with_children! [$name => {
            $($graph_vis $graph_name: $graph_type $(as $tag)?,)*
        }];
    };
    ($(#[$config:meta])* $vis:vis struct $name:ident;) => {
        node!($(#[$config])* $vis struct $name {;});
    }
}
