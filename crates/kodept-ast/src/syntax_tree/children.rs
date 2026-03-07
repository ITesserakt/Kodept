use crate::arity::Arity;
use crate::prelude::ASTNode;
use crate::relationship::NodeRelationship;
use kodept_ecs::component::Component;

#[deprecated]
pub mod arity {
    pub use crate::arity::{Optional, Plural, Singular};
}

pub trait HasChild<Child, Tag = ()>: Family<Tag>
where
    Self: ASTNode,
    Child: ASTNode,
{
}

pub trait Family<Tag = ()>: NodeRelationship<Tag, Self::Arity> + Sized {
    type Arity: Arity;
    type Members: Members<Self, Tag>;
}

pub trait Mapper<Tuple> {
    type Type;
}
pub trait Wrapper {
    type Wrapped<T: Component>;
}

pub struct IdentityMapper;
impl Wrapper for IdentityMapper {
    type Wrapped<T: Component> = T;
}

pub type MembersOf<Node, Tag, Mapper = IdentityMapper> =
    <<Node as Family<Tag>>::Members as Members<Node, Tag>>::Map<Mapper>;
pub trait Members<Parent, Tag> {
    type Map<M: Mapper<Self>>;
}
pub enum Nothing {}

impl<Parent, Tag> Members<Parent, Tag> for Nothing {
    type Map<M: Mapper<Self>> = Nothing;
}

macro_rules! impl_for_tuple {
    ($($t:ident$(,)?)+) => {
        impl<$($t, )+ Parent, Tag> Members<Parent, Tag> for ($($t, )+)
        where
            $(
                Parent: HasChild<$t, Tag>,
                $t: ASTNode,
            )+
        {
            type Map<M: Mapper<Self>> = M::Type;
        }
    };
}

impl_for_tuple!(A);
impl_for_tuple!(A, B);
impl_for_tuple!(A, B, C);
impl_for_tuple!(A, B, C, D);
impl_for_tuple!(A, B, C, D, E);
impl_for_tuple!(A, B, C, D, E, F);
impl_for_tuple!(A, B, C, D, E, F, G);
impl_for_tuple!(A, B, C, D, E, F, G, H);
impl_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M0);

macro_rules! impl_mapper_for_tuple {
    ($($t:ident$(,)?)+) => {
        impl<$($t, )+ W> Mapper<($($t, )+)> for W
        where
            W: Wrapper,
            $($t: Component, )+
        {
            type Type = ($(W::Wrapped<$t>, )+);
        }
    };
}

impl_mapper_for_tuple!(A);
impl_mapper_for_tuple!(A, B);
impl_mapper_for_tuple!(A, B, C);
impl_mapper_for_tuple!(A, B, C, D);
impl_mapper_for_tuple!(A, B, C, D, E);
impl_mapper_for_tuple!(A, B, C, D, E, F);
impl_mapper_for_tuple!(A, B, C, D, E, F, G);
impl_mapper_for_tuple!(A, B, C, D, E, F, G, H);
impl_mapper_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_mapper_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_mapper_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_mapper_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_mapper_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
