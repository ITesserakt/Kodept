#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt::{Enum, Struct, TopLevelNode};
use kodept_core::structure::span::CodeHolder;

use crate::graph::NodeId;
use crate::graph::{SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;
use crate::{node, BodyFnDecl, TyName, TyParam, node_sub_enum};

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum EnumKind {
    Stack,
    Heap,
}

node_sub_enum! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub enum TopLevel {
        Enum(EnumDecl),
        Struct(StructDecl),
        Fn(BodyFnDecl)
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct StructDecl {
        pub name: String,;
        pub parameters: Vec<TyParam>,
        pub contents: Vec<BodyFnDecl>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct EnumDecl {
        pub kind: EnumKind,
        pub name: String,;
        pub contents: Vec<TyName>,
    }
}

impl PopulateTree for Struct {
    type Output = StructDecl;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let node = StructDecl::uninit(context.get_chunk_located(&self.id).to_string());
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
    type Output = EnumDecl;

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        let (kind, name, rest) = match self {
            Enum::Stack { id, contents, .. } => (EnumKind::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumKind::Heap, id, contents),
        };
        let node = EnumDecl::uninit(kind, context.get_chunk_located(name).to_string());
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

    fn convert(
        &self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker + CodeHolder),
    ) -> NodeId<Self::Output> {
        match self {
            TopLevelNode::Enum(x) => x.convert(builder, context).cast(),
            TopLevelNode::Struct(x) => x.convert(builder, context).cast(),
            TopLevelNode::BodiedFunction(x) => x.convert(builder, context).cast(),
        }
    }
}
