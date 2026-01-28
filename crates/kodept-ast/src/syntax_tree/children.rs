use crate::arity::Arity;
use crate::prelude::ASTNode;
use crate::relationship::NodeRelationship;

#[deprecated]
pub mod arity {
    pub use crate::arity::{Optional, Plural, Singular};
}

pub trait HasChild<Child, Tag>: Family<Tag>
where
    Self: ASTNode,
    Child: ASTNode,
{
}

pub trait Family<Tag = ()>: NodeRelationship<Tag, Self::Arity> {
    type Arity: Arity;
}
