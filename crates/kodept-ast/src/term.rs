use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;

use crate::{node, wrapper};
use crate::graph::{GenericASTNode, SyntaxTreeBuilder};
use crate::graph::NodeId;
use crate::traits::Linker;
use crate::traits::PopulateTree;

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper Term {
        reference(Reference) = GenericASTNode::Reference(x) => Some(x),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct Reference {
        pub ident: Identifier,;
    }
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Identifier {
    #[from(ignore)]
    TypeReference { name: String },
    #[from(ignore)]
    Reference { name: String },
}

impl PopulateTree for rlt::Term {
    type Output = Term;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            rlt::Term::Reference(x) => x.convert(builder, context).cast(),
        }
    }
}

impl PopulateTree for rlt::Reference {
    type Output = Reference;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
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
            .add_node(Reference {
                ident,
                id: Default::default(),
            })
            .with_rlt(context, self)
            .id()
    }
}
