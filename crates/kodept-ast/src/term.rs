use crate::graph::traits::PopulateTree;
use crate::graph::SyntaxTree;
use crate::node_id::NodeId;
use crate::traits::{Identifiable, Linker};
use crate::{impl_identifiable_2, Type};
use derive_more::From;
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Reference {
    pub ident: Identifier,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Term {
    Reference(Reference),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ResolvedTypeReference {
    pub name: String,
    pub reference_type: Type,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ResolvedReference {
    pub name: String,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Identifier {
    #[from(ignore)]
    TypeReference {
        name: String,
    },
    #[from(ignore)]
    Reference {
        name: String,
    },
    ResolvedTypeReference(ResolvedTypeReference),
    ResolvedReference(ResolvedReference),
}

impl_identifiable_2! {
    Reference,
    ResolvedReference,
    ResolvedTypeReference
}
node_group!(family: Term, nodes: [Term, Reference]);
node_group!(family: Identifier, nodes: [Identifier, ResolvedTypeReference, ResolvedReference]);

impl Identifiable for Term {
    fn get_id(&self) -> NodeId<Self> {
        match self {
            Term::Reference(x) => x.get_id().cast(),
        }
    }
}

impl PopulateTree for rlt::Term {
    type Output = Term;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
