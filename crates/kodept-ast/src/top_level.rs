use derive_more::{From, Into};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt::{Enum, Struct, TopLevelNode};
use kodept_core::structure::span::CodeHolder;

use crate::graph::NodeId;
use crate::graph::{GenericASTNode, SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;
use crate::{node, wrapper, BodiedFunctionDeclaration, TypeName, TypedParameter};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum EnumKind {
    Stack,
    Heap,
}

wrapper! {
    #[derive(Debug, PartialEq, From, Into)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub wrapper TopLevel {
        enum(EnumDeclaration) = GenericASTNode::Enum(x) => Some(x),
        struct(StructDeclaration) = GenericASTNode::Struct(x) => Some(x),
        function(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x),
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct StructDeclaration {
        pub name: String,;
        pub parameters: Vec<TypedParameter>,
        pub contents: Vec<BodiedFunctionDeclaration>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct EnumDeclaration {
        pub kind: EnumKind,
        pub name: String,;
        pub contents: Vec<TypeName>,
    }
}

impl PopulateTree for Struct {
    type Output = StructDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = StructDeclaration {
            name: context.get_chunk_located(&self.id).to_string(),
            id: Default::default(),
        };
        builder
            .add_node(node)
            .with_children_from(self.body.iter().flat_map(|x| x.inner.as_ref()), context)
            .with_children_from(
                self.parameters.iter().flat_map(|x| x.inner.as_ref()),
                context,
            )
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for Enum {
    type Output = EnumDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
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
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            TopLevelNode::Enum(x) => x.convert(builder, context).cast(),
            TopLevelNode::Struct(x) => x.convert(builder, context).cast(),
            TopLevelNode::BodiedFunction(x) => x.convert(builder, context).cast(),
        }
    }
}
