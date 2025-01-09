use crate::properties::{Param, Type};
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, Str};
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_core::structure::rlt;
use kodept_core::structure::rlt::{Parameter, Tuple};

#[derive(Debug, PartialEq, Component)]
pub struct TyName {
    pub name: Str,
}

#[derive(Debug, PartialEq, Component)]
pub struct ProdTy;

#[derive(Debug, PartialEq, Component)]
pub struct TyParam {
    pub name: Str,
}

#[derive(Debug, PartialEq, Component)]
pub struct NonTyParam {
    pub name: Str,
}

derive_node!(TyName);
derive_node!(ProdTy {
    relations = [
        children TyName where tag = Type,
        children ProdTy where tag = Type,
    ],
    properties = []
});
derive_node!(TyParam {
    relations = [
        optional TyName where tag = Type,
        optional ProdTy where tag = Type,
    ],
    properties = []
});
derive_node!(NonTyParam);

impl FromSyntax for TyName {
    type Syntax = rlt::new_types::TypeName;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(pool, TyName { name })
    }
}

impl FromSyntax for NonTyParam {
    type Syntax = rlt::UntypedParameter;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, NonTyParam { name })
    }
}

impl FromSyntax for TyParam {
    type Syntax = rlt::TypedParameter;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, TyParam { name }).with_children(source, pool, move |scope| {
            scope.choose(Unit, [&node.parameter_type])
        })
    }
}

impl FromSyntax for ProdTy {
    type Syntax = Tuple;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, ProdTy).with_children(source, pool, move |scope| {
            scope.choose(Unit, &node.0.inner)
        })
    }
}

impl<R> Choose<rlt::Type, R, Type> for Unit
where
    R: HasChild<TyName, Type>,
    R: HasChild<ProdTy, Type>,
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &rlt::Type) -> ChildrenDisjoint<R, Source, Type> {
        match node {
            rlt::Type::Reference(x) => ChildrenDisjoint::new::<TyName>(x),
            rlt::Type::Tuple(x) => ChildrenDisjoint::new::<ProdTy>(x),
        }
    }
}

impl<R> Choose<Parameter, R, Param> for Unit
where 
    R: HasChild<TyParam, Param>,
    R: HasChild<NonTyParam, Param>
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &Parameter) -> ChildrenDisjoint<R, Source, Param> {
        match node {
            Parameter::Typed(x) => ChildrenDisjoint::new::<TyParam>(x),
            Parameter::Untyped(x) => ChildrenDisjoint::new::<NonTyParam>(x)
        }
    }
}
