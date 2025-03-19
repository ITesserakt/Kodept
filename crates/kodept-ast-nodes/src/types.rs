use crate::Unit;
use kodept_ast::derive_node;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::tags::Tagged;
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

derive_node!(Params {
    relations = [
        children TyParam,
        children NonTyParam,
    ],
    properties = []
});

derive_node!(TyParams {
    relations = [children TyParam,],
    properties = []
});

derive_node!(Ty {
    relations = [],
    properties = [require Name,]
});
derive_node!(ProdTy {
    relations = [
        children Ty,
        children ProdTy,
    ],
    properties = []
});
derive_node!(TyParam {
    relations = [
        optional Ty,
        optional ProdTy,
    ],
    properties = [require Name,]
});
derive_node!(NonTyParam {
    relations = [],
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

impl<R, Tag> Choose<rlt::Type, R, Tag> for Unit
where
    R: HasChild<Ty, Tag>,
    R: HasChild<ProdTy, Tag>,
    Tag: Tagged,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &rlt::Type) -> ChildrenDisjoint<R, Source, Tag> {
        match node {
            rlt::Type::Reference(x) => ChildrenDisjoint::new::<Ty>(x),
            rlt::Type::Tuple(x) => ChildrenDisjoint::new::<ProdTy>(x),
        }
    }
}

impl<R, Tag> Choose<Parameter, R, Tag> for Unit
where
    R: HasChild<TyParam, Tag>,
    R: HasChild<NonTyParam, Tag>,
    Tag: Tagged,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Parameter) -> ChildrenDisjoint<R, Source, Tag> {
        match node {
            Parameter::Typed(x) => ChildrenDisjoint::new::<TyParam>(x),
            Parameter::Untyped(x) => ChildrenDisjoint::new::<NonTyParam>(x),
        }
    }
}
