use crate::types::Ty;
use crate::Dispatcher;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use kodept_ast::experimental::{AstBuilder, Dispatch, DispatchContext};
use kodept_ast::prelude::CodeHolder;
use kodept_ast::properties::SourceSpan;
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::{Buffer, SpawnedIn};
use kodept_ast::{derive_node, Str};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::Term;
use std::borrow::Cow;
use std::convert::Infallible;

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone)]
pub struct ReferenceContext {
    pub global: bool,
    pub items: Vec<Str>,
}

#[derive(Debug, PartialEq, Component)]
pub struct Ref {
    pub context: ReferenceContext,
    pub ident: Str,
}

derive_node!(Ref);

impl ReferenceContext {
    pub fn empty(global: bool) -> Self {
        Self {
            global,
            items: vec![],
        }
    }

    pub fn global(items: impl IntoIterator<Item: Into<Cow<'static, str>>>) -> Self {
        Self {
            global: true,
            items: items.into_iter().map(|it| Str::from(it.into())).collect(),
        }
    }

    pub fn local(items: impl IntoIterator<Item: Into<Cow<'static, str>>>) -> Self {
        Self {
            global: false,
            items: items.into_iter().map(|it| Str::from(it.into())).collect(),
        }
    }

    /// Means that this context has no items in it and it is local
    pub const fn is_empty_local_context(&self) -> bool {
        !self.global && self.items.is_empty()
    }

    /// Means that this context has no items in it and it is global
    pub const fn is_empty_global_context(&self) -> bool {
        self.global && self.items.is_empty()
    }
}

impl<'a, R, T, A> Dispatch<'a, R, T, A> for Dispatcher<'a, Term>
where
    R: HasChild<Ref, T, Arity = A>,
    R: HasChild<Ty, T, Arity = A>,
    T: Send + Sync + 'static,
    A: kodept_ast::arity::Arity,
{
    type Node = Term;
    type Error = Infallible;

    fn dispatch<B: Buffer>(
        self,
        spawner: DispatchContext<B, R, T, A>,
        source: impl CodeHolder,
    ) -> Result<Entity, Self::Error> {
        match self.0 {
            Term::Reference(x) => {
                let ident = source.get_chunk_located(x);
                let value = Ref {
                    context: ReferenceContext::empty(false),
                    ident,
                };
                Ok(AstBuilder::new(value)
                    .with_property(SourceSpan(x.bounds()))
                    .spawn_in((spawner, x))
                    .finish_any())
            }
            Term::ContextualReference(x) => {
                let ident = source.get_chunk_located(&x.inner);
                let context = (&x.context, source).into();
                let value = Ref { context, ident };

                Ok(AstBuilder::new(value)
                    .with_property(SourceSpan(x.bounds()))
                    .spawn_in((spawner, self.0))
                    .finish_any())
            }
            Term::Constant(x) => {
                let ident = source.get_chunk_located(x);
                let value = Ty {
                    context: ReferenceContext::empty(false),
                    ident,
                };
                Ok(AstBuilder::new(value)
                    .with_property(SourceSpan(x.bounds()))
                    .spawn_in((spawner, x))
                    .finish_any())
            }
            Term::ContextualConstant(x) => {
                let ident = source.get_chunk_located(&x.inner);
                let context = (&x.context, source).into();
                let value = Ty { context, ident };

                Ok(AstBuilder::new(value)
                    .with_property(SourceSpan(x.bounds()))
                    .spawn_in((spawner, self.0))
                    .finish_any())
            }
        }
    }
}
