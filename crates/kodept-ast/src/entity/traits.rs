use crate::prelude::{ASTNode, AnyNodeRefItem, NodeRef};

pub trait FromEnum<'a>: Sized {
    fn from_enum(value: AnyNodeRefItem<'a, '_>) -> Option<Self>;
}

impl<'a, T: ASTNode> FromEnum<'a> for NodeRef<'a, &'a T> {
    #[inline(always)]
    fn from_enum(value: AnyNodeRefItem<'a, '_>) -> Option<Self> {
        value.get()
    }
}

pub trait IntoEnum<T> {
    fn into_enum(self) -> Option<T>;
}

impl<'a, T> IntoEnum<T> for AnyNodeRefItem<'a, '_>
where
    T: FromEnum<'a>,
{
    #[inline(always)]
    fn into_enum(self) -> Option<T> {
        T::from_enum(self)
    }
}
