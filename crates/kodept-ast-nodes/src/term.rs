use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::experimental::ASTBuilder;
use kodept_ast::{derive_node, Str};
use kodept_rlt::prelude::{ContextualReference, Reference, Term};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone)]
pub struct ReferenceContext {
    pub global: bool,
    pub items: Vec<Str>,
}

#[derive(Debug, PartialEq)]
pub enum Identifier {
    TypeReference { name: Str },
    Reference { name: Str },
}

#[derive(Debug, PartialEq, Component)]
pub struct Ref {
    pub context: ReferenceContext,
    pub ident: Identifier,
}

derive_node!(Ref);

impl ReferenceContext {
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
}

impl Identifier {
    pub fn name(&self) -> &str {
        match self {
            Identifier::TypeReference { name, .. } => name,
            Identifier::Reference { name, .. } => name,
        }
    }
}

impl FromSyntax<Term> for Ref {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Term, source: impl CodeHolder) -> Self::Bundle {
        let ident = match node {
            Term::Reference(Reference::Type(x)) => Identifier::TypeReference {
                name: source.get_chunk_located(x),
            },
            Term::Reference(Reference::Identifier(x)) => Identifier::Reference {
                name: source.get_chunk_located(x),
            },
            Term::Contextual(ContextualReference {
                inner: Reference::Type(x),
                ..
            }) => Identifier::TypeReference {
                name: source.get_chunk_located(x),
            },
            Term::Contextual(ContextualReference {
                inner: Reference::Identifier(x),
                ..
            }) => Identifier::Reference {
                name: source.get_chunk_located(x),
            },
        };
        let context = match node {
            Term::Reference(_) => ReferenceContext::default(),
            Term::Contextual(ContextualReference { context, .. }) => {
                let (from_root, refs) = context.clone().unfold();
                ReferenceContext {
                    global: from_root.is_some(),
                    items: refs
                        .into_iter()
                        .map(|it| match it {
                            Reference::Type(x) => source.get_chunk_located(&x),
                            Reference::Identifier(_) => {
                                panic!("Context built with ordinary references is unsupported")
                            }
                        })
                        .collect(),
                }
            }
        };

        ASTBuilder::new(Ref { context, ident }).build()
    }
}
