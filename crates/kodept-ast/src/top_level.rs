use crate::node_id::NodeId;
use crate::traits::{IdProducer, Instantiable, IntoAst, Linker};
use crate::types::{TypeName, TypedParameter};
use crate::{impl_identifiable, BodiedFunctionDeclaration};
use derive_more::From;
use kodept_core::structure::rlt::{Enum, Struct, TopLevelNode};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::node_group;

#[derive(Debug, PartialEq, From)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum TopLevel {
    Enum(EnumDeclaration),
    Struct(StructDeclaration),
    Function(BodiedFunctionDeclaration),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct StructDeclaration {
    pub name: String,
    pub parameters: Vec<TypedParameter>,
    pub rest: Vec<BodiedFunctionDeclaration>,
    id: NodeId<Self>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum EnumKind {
    Stack,
    Heap,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct EnumDeclaration {
    pub name: String,
    pub contents: Vec<TypeName>,
    pub kind: EnumKind,
    id: NodeId<Self>,
}

node_group! {
    family: TopLevel,
    nodes: [
        TopLevel,
        EnumDeclaration,
        StructDeclaration
    ]
}

node_group! {
    family: StructDeclaration,
    nodes: [StructDeclaration, TypedParameter, BodiedFunctionDeclaration]
}

node_group! {
    family: EnumDeclaration,
    nodes: [EnumDeclaration, TypeName]
}

impl_identifiable! {
    StructDeclaration,
    EnumDeclaration
}

impl IntoAst for TopLevelNode {
    type Output = TopLevel;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        match self {
            TopLevelNode::Enum(x) => x.construct(context).into(),
            TopLevelNode::Struct(x) => x.construct(context).into(),
            TopLevelNode::BodiedFunction(x) => x.construct(context).into(),
        }
    }
}

impl IntoAst for Struct {
    type Output = StructDeclaration;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = StructDeclaration {
            name: context.get_chunk_located(&self.id).to_string(),
            parameters: self.parameters.as_ref().map_or(vec![], |it| {
                it.inner.iter().map(|it| it.construct(context)).collect()
            }),
            rest: self.body.as_ref().map_or(vec![], |it| {
                it.inner.iter().map(|it| it.construct(context)).collect()
            }),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for Enum {
    type Output = EnumDeclaration;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let (name, kind, items) = match self {
            Enum::Stack { id, contents, .. } => (
                context.get_chunk_located(id).to_string(),
                EnumKind::Stack,
                contents,
            ),
            Enum::Heap { id, contents, .. } => (
                context.get_chunk_located(id).to_string(),
                EnumKind::Heap,
                contents,
            ),
        };

        let node = EnumDeclaration {
            name,
            contents: items.as_ref().map_or(vec![], |it| {
                it.inner.iter().map(|it| it.construct(context)).collect()
            }),
            kind,
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl Instantiable for StructDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            parameters: self
                .parameters
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            rest: self
                .rest
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for EnumDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            name: self.name.clone(),
            contents: self
                .contents
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            kind: self.kind.clone(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for TopLevel {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        match self {
            TopLevel::Enum(x) => x.new_instance(context).into(),
            TopLevel::Struct(x) => x.new_instance(context).into(),
            TopLevel::Function(x) => x.new_instance(context).into(),
        }
    }
}
