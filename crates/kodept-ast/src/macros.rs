#[macro_export]
macro_rules! derive_node {
    (
        $self:ty {
            properties = [$($modifier:ident $($name:ident)?,)*]
        }
    ) => {
        impl $crate::prelude::ASTNode for $self {}

        $($crate::property!($self => $modifier $($name)?);)+
    };
    ($self:ty) => {
        impl $crate::prelude::ASTNode for $self {}
    }
}

#[macro_export]
macro_rules! relation {
    ($self:ty => or $name:ident($child_arity:tt $child:ty)) => {
        impl $crate::syntax_tree::children::HasChild<$child, $name> for $self {
            type Arity = $crate::arity!($child_arity);
        }
    };
    ($self:ty => either $name:ident($child_arity:tt $child:ty)) => {
        pub struct $name;
        impl $crate::syntax_tree::children::HasChild<$child, $name> for $self {
            type Arity = $crate::arity!($child_arity);
        }
    };
    ($self:ty => $child_arity:tt $child:ty) => {
        impl $crate::syntax_tree::children::HasChild<$child, ()> for $self {
            type Arity = $crate::arity!($child_arity);
        }
    }
}

#[macro_export]
macro_rules! arity {
    (child) => {
        $crate::arity::Singular
    };
    (optional) => {
        $crate::arity::Optional
    };
    (children) => {
        $crate::arity::Plural
    };
}

#[macro_export]
macro_rules! property {
    ($self:ty => $name:ty) => {
        impl $crate::properties::HasProperty<$name> for $self {}
    };
    ($self:ty => require $name:ty) => {
        impl $crate::properties::RequireProperty<$name> for $self {}
    };
}
