use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thin_vec::ThinVec;

use crate::graph::SubSyntaxTree;
use crate::interning::SharedStr;
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
    items: ThinVec<SharedStr>,
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
    TypeReference { name: SharedStr },
    #[from(ignore)]
    Reference { name: SharedStr },
}

impl ReferenceContext {
    pub fn global(items: impl IntoIterator<Item: Into<String>>) -> Self {
        Self {
            global: true,
            items: items
                .into_iter()
                .map(|it| it.into())
                .map(|it| SharedStr::from(Cow::Owned(it)))
                .collect(),
        }
    }

    pub fn local(items: impl IntoIterator<Item: Into<String>>) -> Self {
        Self {
            global: false,
            items: items
                .into_iter()
                .map(|it| it.into())
                .map(|it| SharedStr::from(Cow::Owned(it)))
                .collect(),
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Term {
    type Root = Term;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            rlt::Term::Reference(x) => x.convert(context).cast(),
            rlt::Term::Contextual(x) => x.inner.convert(context).cast(),
        }
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::ContextualReference {
    type Root = Ref;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
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

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
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
