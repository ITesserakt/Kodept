use crate::arity::Arity;
use crate::prelude::{ASTNode, CodeHolder, FromSyntax};
use crate::relationship::NodeRelationship;
use crate::resource::rlt::SyntaxVariant;
use crate::syntax_tree::prelude::{ASTBuilder, Pool};
use std::fmt::Debug;
use std::marker::PhantomData;

#[deprecated]
pub mod arity {
    pub use crate::arity::{Optional, Plural, Singular};
}

pub trait HasChild<Child, Tag>: NodeRelationship<Child, Tag>
where
    Self: ASTNode,
    Child: ASTNode,
{
    type Arity: Arity;
}

type DynFromSyntax<'w, Source> = dyn FnOnce(SyntaxVariant<'w>, Source, Pool<'w>) -> ASTBuilder<()>;

pub struct ChildrenDisjoint<'p, Root, Source, Arity, Tag = ()>
where
    Source: CodeHolder,
{
    inner: SyntaxVariant<'p>,
    conversion: Box<DynFromSyntax<'p, Source>>,
    _phantom: PhantomData<(Tag, Root, Arity)>,
}

impl<'w, Root, Source, Arity, Tag> ChildrenDisjoint<'w, Root, Source, Arity, Tag>
where
    Source: CodeHolder,
{
    #[inline(always)]
    pub fn new<U>(node: &'w U::Syntax) -> Self
    where
        &'w U::Syntax: Into<SyntaxVariant<'w>>,
        SyntaxVariant<'w>: TryInto<&'w U::Syntax, Error: Debug>,
        Root: HasChild<U, Tag, Arity = Arity>,
        U: FromSyntax + ASTNode,
    {
        Self::ad_hoc(node, |node, source, pool| {
            U::from_syntax(node, source, pool)
        })
    }

    #[inline(always)]
    #[allow(unsafe_code)]
    pub(crate) fn call(self, source: Source, pool: Pool<'w>) -> ASTBuilder<()> {
        let part = (self.conversion)(self.inner, source, pool);
        unsafe {
            pool.link_syntax(part.id(), self.inner);
        }
        part
    }

    #[inline(always)]
    pub fn ad_hoc<'a, T, U>(
        node: &'a T,
        f: impl FnOnce(&'w T, Source, Pool<'w>) -> ASTBuilder<U> + 'static,
    ) -> Self
    where
        &'a T: Into<SyntaxVariant<'w>>,
        SyntaxVariant<'w>: TryInto<&'w T, Error: Debug>,
        Root: HasChild<U, Tag, Arity = Arity>,
        U: ASTNode,
        T: 'w,
    {
        Self {
            inner: node.into(),
            conversion: Box::new(move |node, source, pool| {
                let node = node.try_into().unwrap();
                f(node, source, pool).erase()
            }),
            _phantom: PhantomData,
        }
    }
}
