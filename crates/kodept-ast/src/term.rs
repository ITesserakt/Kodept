use crate::node_id::NodeId;
use crate::traits::{IdProducer, Identifiable, Instantiable, IntoAst, Linker};
use crate::{impl_identifiable, Type};
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

impl_identifiable! {
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

impl IntoAst for rlt::Term {
    type Output = Term;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = match self {
            rlt::Term::Reference(x) => {
                let node = Reference {
                    ident: x.construct(context),
                    id: context.next_id(),
                };
                context.link(node, x).into()
            }
        };
        context.link(node, self)
    }
}

impl IntoAst for rlt::Reference {
    type Output = Identifier;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        match self {
            rlt::Reference::Type(x) => Identifier::TypeReference {
                name: context.get_chunk_located(x).to_string(),
            },
            rlt::Reference::Identifier(x) => Identifier::Reference {
                name: context.get_chunk_located(x).to_string(),
            },
        }
    }
}

impl Instantiable for Term {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Term::Reference(x) => x.new_instance(context).into(),
        }
    }
}

impl Instantiable for Reference {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            ident: self.ident.new_instance(context),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for Identifier {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            Identifier::TypeReference { name } => Identifier::TypeReference { name: name.clone() },
            Identifier::Reference { name } => Identifier::Reference { name: name.clone() },
            Identifier::ResolvedTypeReference(x) => x.new_instance(context).into(),
            Identifier::ResolvedReference(x) => x.new_instance(context).into(),
        }
    }
}

impl Instantiable for ResolvedReference {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for ResolvedTypeReference {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            reference_type: todo!(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}
