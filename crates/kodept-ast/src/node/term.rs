use derive_more::From;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::SubSyntaxTree;
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum};

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub enum Term {
        Ref(Ref)
    }
}

#[derive(Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ReferenceContext {
    global: bool,
    items: Vec<String>,
}

node! {
    #[derive(Debug, PartialEq)]
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
    TypeReference { name: String },
    #[from(ignore)]
    Reference { name: String },
}

impl ReferenceContext {
    pub fn global(items: impl IntoIterator<Item: Into<String>>) -> Self {
        Self {
            global: true,
            items: items.into_iter().map(|it| it.into()).collect(),
        }
    }

    pub fn local(items: impl IntoIterator<Item: Into<String>>) -> Self {
        Self {
            global: false,
            items: items.into_iter().map(|it| it.into()).collect(),
        }
    }
}

impl PopulateTree for rlt::Term {
    type Root = Term;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        match self {
            rlt::Term::Reference(x) => x.convert(context).cast(),
            rlt::Term::Contextual(x) => x.inner.convert(context).cast(),
        }
    }
}

impl PopulateTree for rlt::ContextualReference {
    type Root = Ref;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let ident = match &self.inner {
            rlt::Reference::Type(x) => Identifier::TypeReference {
                name: context.get_chunk_located(x).to_string(),
            },
            rlt::Reference::Identifier(x) => Identifier::Reference {
                name: context.get_chunk_located(x).to_string(),
            },
        };
        let (from_root, refs) = self.context.clone().unfold();
        let ctx = ReferenceContext {
            global: from_root.is_some(),
            items: refs
                .into_iter()
                .map(|it| match it {
                    rlt::Reference::Type(x) => context.get_chunk_located(&x).to_string(),
                    rlt::Reference::Identifier(_) => {
                        panic!("Context built with ordinary references is unsupported")
                    }
                })
                .collect(),
        };
        SubSyntaxTree::new(Ref::uninit(ctx, ident).with_rlt(self))
    }
}

impl PopulateTree for rlt::Reference {
    type Root = Ref;

    fn convert(&self, context: &impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let ident = match self {
            rlt::Reference::Type(x) => Identifier::TypeReference {
                name: context.get_chunk_located(x).to_string(),
            },
            rlt::Reference::Identifier(x) => Identifier::Reference {
                name: context.get_chunk_located(x).to_string(),
            },
        };
        SubSyntaxTree::new(Ref::uninit(Default::default(), ident).with_rlt(self))
    }
}
