use crate::Unit;
use kodept_ast::{derive_node, relation};
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_rlt::new_types;
use kodept_rlt::prelude as rlt;
use kodept_rlt::prelude::{Parameter, Tuple, TypedParameter, UntypedParameter};

#[derive(Debug, PartialEq, Component)]
pub struct Ty;

#[derive(Debug, PartialEq, Component)]
pub struct ProdTy;

#[derive(Debug, PartialEq, Component)]
pub struct TyParam;

#[derive(Debug, PartialEq, Component)]
pub struct NonTyParam;

#[derive(Debug, PartialEq, Component)]
pub struct Params;
#[derive(Debug, PartialEq, Component)]
pub struct TyParams;

derive_node!(Params);
relation!(Params => children TyParam);
relation!(Params => children NonTyParam);

derive_node!(TyParams);
relation!(TyParams => children TyParam);

derive_node!(Ty {
    properties = [require Name,]
});

derive_node!(ProdTy);
relation!(ProdTy => children Ty);
relation!(ProdTy => children ProdTy);

derive_node!(TyParam {
    properties = [require Name,]
});
relation!(TyParam => optional Ty);
relation!(TyParam => optional ProdTy);

derive_node!(NonTyParam {
    properties = [require Name,]
});

impl FromSyntax for Ty {
    type Syntax = new_types::TypeName;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(pool, Ty).with_property(Name(name))
    }
}

impl FromSyntax for NonTyParam {
    type Syntax = UntypedParameter;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, NonTyParam).with_property(Name(name))
    }
}

impl FromSyntax for TyParam {
    type Syntax = TypedParameter;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, TyParam)
            .with_property(Name(name))
            .with_children(source, pool, move |scope| {
                scope.choose(Unit, [&node.parameter_type])
            })
    }
}

impl FromSyntax for ProdTy {
    type Syntax = Tuple;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ProdTy).with_children(source, pool, move |scope| {
            scope.choose(Unit, node.0.inner.as_ref())
        })
    }
}

impl<R, Tag, A> Choose<rlt::Type, R, Tag> for Unit
where
    R: HasChild<Ty, Tag, Arity = A>,
    R: HasChild<ProdTy, Tag, Arity = A>,
    Tag: Send + Sync + 'static,
{
    type Arity = A;

    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &rlt::Type) -> ChildrenDisjoint<R, Source, A, Tag> {
        match node {
            rlt::Type::Reference(x) => ChildrenDisjoint::new::<Ty>(x),
            rlt::Type::Tuple(x) => ChildrenDisjoint::new::<ProdTy>(x),
        }
    }
}

impl<R, Tag, A> Choose<Parameter, R, Tag> for Unit
where
    R: HasChild<TyParam, Tag, Arity = A>,
    R: HasChild<NonTyParam, Tag, Arity = A>,
    Tag: Send + Sync + 'static,
{
    type Arity = A;
    
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Parameter) -> ChildrenDisjoint<R, Source, A, Tag> {
        match node {
            Parameter::Typed(x) => ChildrenDisjoint::new::<TyParam>(x),
            Parameter::Untyped(x) => ChildrenDisjoint::new::<NonTyParam>(x),
        }
    }
}
