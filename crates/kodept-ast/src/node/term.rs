use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::graph::NodeId;
use crate::graph::{GenericASTNode, SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;
use crate::{node, wrapper};

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Term {
        reference(Reference) = GenericASTNode::Reference(x) => x.into(),
    }
}

#[derive(Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ReferenceContext {
    global: bool,
    items: Vec<String>
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Reference {
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
    type Output = Term;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Term::Reference(x) => x.convert(builder, context).cast(),
            rlt::Term::Contextual(x) => x.inner.convert(builder, context).cast()
        }
    }
}

impl PopulateTree for rlt::ContextualReference {
    type Output = Reference;

    fn convert(&self, builder: &mut SyntaxTreeBuilder, context: &mut (impl Linker + CodeHolder)) -> NodeId<Self::Output> {
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
            items: refs.into_iter().map(|it| match it {
                rlt::Reference::Type(x) => context.get_chunk_located(&x).to_string(),
                rlt::Reference::Identifier(_) => panic!("Context built with ordinary references is unsupported")
            }).collect(),
        };
        builder.add_node(Reference::uninit(ctx, ident))
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for rlt::Reference {
    type Output = Reference;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let ident = match self {
            rlt::Reference::Type(x) => Identifier::TypeReference {
                name: context.get_chunk_located(x).to_string(),
            },
            rlt::Reference::Identifier(x) => Identifier::Reference {
                name: context.get_chunk_located(x).to_string(),
            },
        };
        builder
            .add_node(Reference::uninit(Default::default(), ident))
            .with_rlt(context, self)
            .id()
    }
}
