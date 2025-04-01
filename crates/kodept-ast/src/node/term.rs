use std::borrow::Cow;
use derive_more::From;
use kodept_rlt::prelude as rlt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::graph::SubSyntaxTree;
use crate::traits::{CodeHolder, PopulateTree};
use crate::{node, node_sub_enum, Str};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum Term {
        Ref(Ref)
    }
}

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ReferenceContext {
    pub global: bool,
    pub items: Vec<Str>,
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Ref {
        pub context: ReferenceContext,
        pub ident: Identifier,;
    }
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Identifier {
    #[from(ignore)]
    TypeReference { name: Str },
    #[from(ignore)]
    Reference { name: Str },
}

impl ReferenceContext {
    pub fn global(items: impl IntoIterator<Item: Into<Cow<'static, str>>>) -> Self {
        Self {
            global: true,
            items: items
                .into_iter()
                .map(|it| it.into())
                .collect(),
        }
    }

    pub fn local(items: impl IntoIterator<Item: Into<Cow<'static, str>>>) -> Self {
        Self {
            global: false,
            items: items
                .into_iter()
                .map(|it| it.into())
                .collect(),
        }
    }
}

impl Identifier {
    pub fn name(&self) -> &Str {
        match self {
            Identifier::TypeReference { name, .. } => name,
            Identifier::Reference { name, .. } => name
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Term {
    type Root = Term;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            rlt::Term::Reference(x) => x.convert(context).cast(),
            rlt::Term::Contextual(x) => x.convert(context).cast(),
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::ContextualReference {
    type Root = Ref;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let ident = match &self.inner {
            rlt::Reference::Type(x) => Identifier::TypeReference {
                name: context.get_chunk_located(x),
            },
            rlt::Reference::Identifier(x) => Identifier::Reference {
                name: context.get_chunk_located(x),
            },
        };
        let (from_root, refs) = self.context.clone().unfold();
        let ctx = ReferenceContext {
            global: from_root.is_some(),
            items: refs
                .into_iter()
                .map(|it| match it {
                    rlt::Reference::Type(x) => context.get_chunk_located(&x),
                    rlt::Reference::Identifier(_) => {
                        panic!("Context built with ordinary references is unsupported")
                    }
                })
                .collect(),
        };
        SubSyntaxTree::new(Ref::uninit(ctx, ident).with_rlt(self))
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Reference {
    type Root = Ref;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let ident = match self {
            rlt::Reference::Type(x) => Identifier::TypeReference {
                name: context.get_chunk_located(x),
            },
            rlt::Reference::Identifier(x) => Identifier::Reference {
                name: context.get_chunk_located(x),
            },
        };
        SubSyntaxTree::new(Ref::uninit(Default::default(), ident).with_rlt(self))
    }
}
