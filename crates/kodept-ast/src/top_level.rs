use crate::graph::graph::SyntaxTree;
use crate::graph::traits::PopulateTree;
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{
    impl_identifiable_2, with_children, BodiedFunctionDeclaration, TypeName, TypedParameter,
};
use derive_more::From;
use kodept_core::structure::rlt::{Enum, Struct, TopLevelNode};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

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
    id: NodeId<StructDeclaration>,
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
    pub kind: EnumKind,
    pub name: String,
    id: NodeId<EnumDeclaration>,
}

impl_identifiable_2! { StructDeclaration, EnumDeclaration }

with_children!(StructDeclaration => {
    pub parameters: Vec<TypedParameter>
});

with_children!(EnumDeclaration => {
    pub contents: Vec<TypeName>
});

impl PopulateTree for Struct {
    type Output = StructDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = StructDeclaration {
            name: context.get_chunk_located(&self.id).to_string(),
            id: Default::default(),
        };
        builder.add_node(node).with_rlt(context, self).id()
    }
}

impl PopulateTree for Enum {
    type Output = EnumDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let (kind, name, rest) = match self {
            Enum::Stack { id, contents, .. } => (EnumKind::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumKind::Heap, id, contents),
        };
        let node = EnumDeclaration {
            kind,
            name: context.get_chunk_located(name).to_string(),
            id: Default::default(),
        };
        builder
            .add_node(node)
            .with_children_from(
                rest.as_ref()
                    .map_or([].as_slice(), |x| x.inner.iter().as_slice()),
                context,
            )
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for TopLevelNode {
    type Output = TopLevel;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            TopLevelNode::Enum(x) => x.convert(builder, context).cast(),
            TopLevelNode::Struct(x) => x.convert(builder, context).cast(),
            TopLevelNode::BodiedFunction(x) => todo!(),
        }
    }
}
