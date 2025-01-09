use crate::prelude::{ASTNode, CodeHolder, FromSyntax};
use crate::properties::tags::{NoTag, Tagged};
use crate::resource::rlt::SyntaxVariant;
use crate::syntax_tree::children::arity::Arity;
use crate::syntax_tree::prelude::{ASTBuilder, Pool};
use std::fmt::Debug;
use std::marker::PhantomData;

pub mod arity {
    use sealed::sealed;

    #[sealed]
    pub trait Arity {}

    /// Describes that parent must have single child of that type
    pub struct Singlular;
    #[sealed]
    impl Arity for Singlular {}

    /// Describes that parent may not have any child of that type
    pub struct Optional;
    #[sealed]
    impl Arity for Optional {}

    /// Describes that parent may have multiple children of that type
    pub struct Plural;
    #[sealed]
    impl Arity for Plural {}
}

pub trait HasChild<Child, Tag = NoTag>
where
    Self: ASTNode,
    Child: ASTNode,
    Tag: Tagged,
{
    type Arity: Arity;
}

type DynFromSyntax<'p, Source> = dyn FnOnce(SyntaxVariant<'p>, Source, &'p Pool) -> ASTBuilder<()>;

pub struct ChildrenDisjoint<'p, Root, Source, Tag>
where
    Source: CodeHolder,
{
    pub(crate) inner: SyntaxVariant<'p>,
    pub(crate) conversion: Box<DynFromSyntax<'p, Source>>,
    pub(crate) _phantom: PhantomData<(Tag, Root)>,
}

impl<'p, Root, Source, Tag> ChildrenDisjoint<'p, Root, Source, Tag>
where
    Source: CodeHolder,
    Tag: Tagged,
{
    #[inline(always)]
    pub fn new<'a, U>(node: &'a U::Syntax) -> Self
    where
        &'a U::Syntax: Into<SyntaxVariant<'p>>,
        SyntaxVariant<'p>: TryInto<&'p U::Syntax, Error: Debug>,
        Root: HasChild<U, Tag>,
        U: FromSyntax + ASTNode,
    {
        Self {
            inner: node.into(),
            conversion: Box::new(|node, source, pool| {
                let node = node.try_into().unwrap();
                U::from_syntax(node, source, pool).erase()
            }),
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn ad_hoc<'a, T, U>(
        node: &'a T,
        f: impl FnOnce(&'p T, Source, &'p Pool) -> ASTBuilder<U> + 'static,
    ) -> Self
    where
        &'a T: Into<SyntaxVariant<'p>>,
        SyntaxVariant<'p>: TryInto<&'p T, Error: Debug>,
        Root: HasChild<U, Tag>,
        U: ASTNode,
        T: 'p,
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
