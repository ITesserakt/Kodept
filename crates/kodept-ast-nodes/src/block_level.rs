use crate::code_flow::IfExpr;
use crate::consts::Const;
use crate::expression::{App, BinExpr, Exprs, Lambda};
use crate::literal::{Literal, Tuple};
use crate::term::Ref;
use crate::types::{ProdTy, Ty};
use crate::Dispatcher;
use bevy_ecs::prelude::Component;
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, Dispatch, DispatchContext, FromSyntax, SpawnContext};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::SpawnedIn;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::{BlockLevelNode, Body, InitializedVariable, Variable};

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
relation!(InitVar => optional Ref);
relation!(InitVar => optional Ty);
relation!(InitVar => optional Literal);
relation!(InitVar => optional Tuple);

impl FromSyntax<Variable> for VarDecl {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &Variable,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (kind, id, ty) = match node {
            Variable::Immutable {
                id, assigned_type, ..
            } => (VarDecl::Immutable, id, assigned_type),
            Variable::Mutable {
                id, assigned_type, ..
            } => (VarDecl::Mutable, id, assigned_type),
        };
        let name = source.get_chunk_located(id);
        let mut builder = AstBuilder::new(kind)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner);
        if let Some((_, ty)) = ty {
            builder.with_dispatch::<Dispatcher<_>, _, _>(ty, source)?;
        }
        Ok(builder.finish())
    }
}

impl FromSyntax<InitializedVariable> for InitVar {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &InitializedVariable,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        Ok(AstBuilder::new(InitVar)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner)
            .with_child::<_, VarDecl, _>(&node.variable, source)?
            .with_dispatch::<Dispatcher<_>, _, _>(&node.expression, source)?
            .finish())
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Body>
where
    R: HasChild<Exprs, T, Arity = A>,
    T: Send + Sync + 'static,
    A: kodept_ast::arity::Arity,
{
    type Node = Body;
    type Error = crate::Error;

    fn dispatch(
        self,
        mut spawner: DispatchContext<R, T, A>,
        source: impl CodeHolder,
    ) -> Result<bevy_ecs::entity::Entity, Self::Error> {
        match self.0 {
            Body::Block(x) => spawner.forward::<_, Exprs>(x, source),
            Body::Simplified { expression, .. } => AstBuilder::new(Exprs)
                .with_property(SourceSpan(expression.bounds()))
                .spawn_in((spawner, expression))
                .with_dispatch::<Dispatcher<_>, _, _>(expression, source)
                .map(|it| it.finish_any()),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, BlockLevelNode>
where
    R: HasChild<Exprs, T, Arity = A>,
    R: HasChild<InitVar, T, Arity = A>,
    R: HasChild<Const, T, Arity = A>,
    R: HasChild<BinExpr, T, Arity = A>,
    R: HasChild<App, T, Arity = A>,
    R: HasChild<Lambda, T, Arity = A>,
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<Tuple, T, Arity = A>,
    R: HasChild<Literal, T, Arity = A>,
    R: HasChild<IfExpr, T, Arity = A>,
    T: Send + Sync + 'static,
    A: kodept_ast::arity::Arity,
{
    type Node = BlockLevelNode;
    type Error = crate::Error;

    fn dispatch(
        self,
        mut spawner: DispatchContext<R, T, A>,
        source: impl CodeHolder,
    ) -> Result<bevy_ecs::entity::Entity, Self::Error> {
        match self.0 {
            BlockLevelNode::InitVar(x) => spawner.forward::<_, InitVar>(x, source),
            BlockLevelNode::Block(x) => spawner.forward::<_, Exprs>(x, source),
            BlockLevelNode::Function(x) => spawner.forward::<_, Const>(x, source),
            BlockLevelNode::Operation(x) => spawner.dispatch::<Dispatcher<_>>(x, source),
        }
    }
}
