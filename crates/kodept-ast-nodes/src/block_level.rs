use crate::code_flow::IfExpr;
use crate::constants::Const;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::function::Func;
use crate::literal::{Literal, Tuple};
use crate::term::Ref;
use crate::types::{ProdTy, Ty};
use crate::utils::const_disjoint;
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, relation};
use kodept_ast::arity::Arity;
use kodept_rlt::prelude::{BlockLevelNode, InitializedVariable, Variable};

#[derive(Debug, PartialEq, Component)]
pub enum VarDecl {
    Immutable,
    Mutable,
}

#[derive(Debug, PartialEq, Component)]
pub struct InitVar;

derive_node!(VarDecl {
    properties = [require Name,]
});
relation!(VarDecl => optional Ty);
relation!(VarDecl => optional ProdTy);

derive_node!(InitVar);
relation!(InitVar => child VarDecl);
relation!(InitVar => optional Exprs);
relation!(InitVar => optional App);
relation!(InitVar => optional Lambda);
relation!(InitVar => optional IfExpr);
relation!(InitVar => optional BinExpr);
relation!(InitVar => optional UnExpr);
relation!(InitVar => optional Ref);
relation!(InitVar => optional Literal);
relation!(InitVar => optional Tuple);

impl FromSyntax for VarDecl {
    type Syntax = Variable;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        let (kind, id, ty) = match node {
            Variable::Immutable {
                id, assigned_type, ..
            } => (VarDecl::Immutable, id, assigned_type),
            Variable::Mutable {
                id, assigned_type, ..
            } => (VarDecl::Mutable, id, assigned_type),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(pool, kind)
            .with_property(Name(name))
            .with_children(source, pool, move |scope| {
                scope.maybe_choose(Unit, ty.as_ref().map(|it| [&it.1]));
            })
    }
}

impl FromSyntax for InitVar {
    type Syntax = InitializedVariable;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, InitVar).with_children(source, pool, |scope| {
            scope.many::<VarDecl, _>([&node.variable]);
            scope.choose(Unit, [&node.expression]);
        })
    }
}

impl<R, A> Choose<BlockLevelNode, R, ()> for Unit
where
    A: Arity,
    R: HasChild<InitVar, (), Arity = A>,
    R: HasChild<Const, (), Arity = A>,
    R: HasChild<Exprs, (), Arity = A>,
    R: HasChild<App, (), Arity = A>,
    R: HasChild<Lambda, (), Arity = A>,
    R: HasChild<IfExpr, (), Arity = A>,
    R: HasChild<BinExpr, (), Arity = A>,
    R: HasChild<UnExpr, (), Arity = A>,
    R: HasChild<Ref, (), Arity = A>,
    R: HasChild<Literal, (), Arity = A>,
    R: HasChild<Tuple, (), Arity = A>,
{
    type Arity = A;
    
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &BlockLevelNode) -> ChildrenDisjoint<R, Source, Self::Arity, ()> {
        match node {
            BlockLevelNode::InitVar(x) => ChildrenDisjoint::new::<InitVar>(x),
            BlockLevelNode::Function(x) => {
                const_disjoint::<Func, _, _, _>(x, |node, source: Source| {
                    source.get_chunk_located(&node.id)
                })
            }
            BlockLevelNode::Operation(x) => Unit::branch(x),
            BlockLevelNode::Block(x) => ChildrenDisjoint::new::<Exprs>(x),
        }
    }
}
