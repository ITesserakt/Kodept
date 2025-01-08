#[macro_export]
macro_rules! derive_node {
    (
        $self:ty {
            relations = [$($child_arity:ident $child:ty$( where tag = $tag:ty)?,)*],
            properties = [$($modifier:ident $($name:ident)?,)*]
        }
    ) => {
        impl $crate::prelude::ASTNode for $self {}
        
        $(
        impl $crate::syntax_tree::children::HasChild<$child, $($tag)?> for $self {
            type Arity = $crate::arity!($child_arity);
        }
        )*
        
        $(
        $crate::property!($self => $modifier $($name)?);
        )*
    };
    ($self:ty) => {
        impl $crate::prelude::ASTNode for $self {}
    }
}

#[macro_export]
macro_rules! arity {
    (child) => { $crate::syntax_tree::children::arity::Singlular };
    (optional) => { $crate::syntax_tree::children::arity::Optional };
    (children) => { $crate::syntax_tree::children::arity::Plural };
}

#[macro_export]
macro_rules! property {
    ($self:ty => $name:ty) => { impl $crate::properties::HasProperty<$name> for $self {} };
    ($self:ty => require $name:ty) => { impl $crate::properties::RequireProperty<$name> for $self {} };
}
