use crate::properties::{Param, Type};
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::{Name, RequireProperty};
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::derive_node;
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

derive_node!(Ty);
derive_node!(ProdTy {
    relations = [
        children Ty where tag = Type,
        children ProdTy where tag = Type,
    ],
    properties = []
});
derive_node!(TyParam {
    relations = [
        optional Ty where tag = Type,
        optional ProdTy where tag = Type,
    ],
    properties = []
});
derive_node!(NonTyParam);

impl FromSyntax for Ty {
    type Syntax = new_types::TypeName;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(pool, Ty).with_property(Name { name })
    }
}

impl FromSyntax for NonTyParam {
    type Syntax = UntypedParameter;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, NonTyParam).with_property(Name { name })
    }
}

impl FromSyntax for TyParam {
    type Syntax = TypedParameter;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, TyParam)
            .with_property(Name { name })
            .with_children(source, pool, move |scope| {
                scope.choose(Unit, [&node.parameter_type])
            })
    }
}

impl FromSyntax for ProdTy {
    type Syntax = Tuple;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ProdTy).with_children(source, pool, move |scope| {
            scope.choose(Unit, node.0.inner.as_ref())
        })
    }
}

impl<R> Choose<rlt::Type, R, Type> for Unit
where
    R: HasChild<Ty, Type>,
    R: HasChild<ProdTy, Type>,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &rlt::Type) -> ChildrenDisjoint<R, Source, Type> {
        match node {
            rlt::Type::Reference(x) => ChildrenDisjoint::new::<Ty>(x),
            rlt::Type::Tuple(x) => ChildrenDisjoint::new::<ProdTy>(x),
        }
    }
}

impl<R> Choose<Parameter, R, Param> for Unit
where
    R: HasChild<TyParam, Param>,
    R: HasChild<NonTyParam, Param>,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Parameter) -> ChildrenDisjoint<R, Source, Param> {
        match node {
            Parameter::Typed(x) => ChildrenDisjoint::new::<TyParam>(x),
            Parameter::Untyped(x) => ChildrenDisjoint::new::<NonTyParam>(x),
        }
    }
}

impl RequireProperty<Name> for Ty {}
impl RequireProperty<Name> for TyParam {}
impl RequireProperty<Name> for NonTyParam {}
