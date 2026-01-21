use crate::term::ReferenceContext;
use crate::Dispatcher;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::relationship::Relationship;
use kodept_ast::experimental::{AstBuilder, Dispatch, FromSyntax};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::{
    Buffer, DispatchContext, GenericSpawnContext, SpawnedIn,
};
use kodept_ast::{derive_node, relation, Str};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types;
use kodept_rlt::prelude::{Parameter, Tuple, TypedParameter, UntypedParameter};
use std::convert::Infallible;

#[derive(Debug, PartialEq, Component)]
pub struct Ty {
    pub context: ReferenceContext,
    pub ident: Str,
}

#[derive(Debug, PartialEq, Component)]
pub struct ProdTy;

#[derive(Debug, PartialEq, Component)]
pub struct TyParam;

#[derive(Debug, PartialEq, Component)]
pub struct NonTyParam;

derive_node!(Ty);

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

impl FromSyntax<new_types::TypeName> for Ty {
    type Error = Infallible;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &new_types::TypeName,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(node);
        Ok(AstBuilder::new(Ty {
            context: ReferenceContext::empty(false),
            ident: name,
        })
        .with_property(SourceSpan(node.0.into()))
        .spawn_in(spawner)
        .finish())
    }
}

impl FromSyntax<UntypedParameter> for NonTyParam {
    type Error = Infallible;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &UntypedParameter,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        Ok(AstBuilder::new(NonTyParam)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner)
            .finish())
    }
}

impl FromSyntax<TypedParameter> for TyParam {
    type Error = Infallible;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &TypedParameter,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        Ok(AstBuilder::new(TyParam)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner)
            .with_dispatch::<Dispatcher<_>, _, _>(&node.parameter_type, source)?
            .finish())
    }
}

impl FromSyntax<Tuple> for ProdTy {
    type Error = Infallible;

    fn from_syntax<B: Buffer, R: Relationship>(
        node: &Tuple,
        spawner: GenericSpawnContext<R, B>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        Ok(AstBuilder::new(ProdTy)
            .with_property(SourceSpan(node.0.left.0 + node.0.right.0))
            .spawn_in(spawner)
            .with_dispatches::<Dispatcher<_>, _, _>(node.0.inner.as_ref(), source)?
            .finish())
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, kodept_rlt::prelude::Type>
where
    R: HasChild<Ty, T, Arity = A>,
    R: HasChild<ProdTy, T, Arity = A>,
    T: Send + Sync + 'static,
    A: kodept_ast::arity::Arity,
{
    type Node = kodept_rlt::prelude::Type;
    type Error = Infallible;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            kodept_rlt::prelude::Type::ContextualReference(context, ident) => {
                let (is_global, context_items) = context.unfold();
                let ident = source.get_chunk_located(ident);
                let context_items = context_items
                    .into_iter()
                    .map(|it| source.get_chunk_located(it));
                let context = if is_global.is_some() {
                    ReferenceContext::global(context_items)
                } else {
                    ReferenceContext::local(context_items)
                };
                Ok(AstBuilder::new(Ty { context, ident })
                    .with_property(SourceSpan(self.0.bounds()))
                    .spawn_in((spawner, self.0))
                    .finish_any())
            }
            kodept_rlt::prelude::Type::Reference(x) => spawner.forward::<_, Ty>(x, source),
            kodept_rlt::prelude::Type::Tuple(x) => spawner.forward::<_, ProdTy>(x, source),
        }
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Parameter>
where
    R: HasChild<TyParam, T, Arity = A>,
    R: HasChild<NonTyParam, T, Arity = A>,
    T: Send + Sync + 'static,
    A: kodept_ast::arity::Arity,
{
    type Node = Parameter;
    type Error = Infallible;

    fn dispatch<B: Buffer>(
        self,
        mut spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            Parameter::Typed(x) => spawner.forward::<_, TyParam>(x, source),
            Parameter::Untyped(x) => spawner.forward::<_, NonTyParam>(x, source),
        }
    }
}
