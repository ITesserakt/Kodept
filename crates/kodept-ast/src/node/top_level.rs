#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_core::structure::rlt::{Enum, Struct, TopLevelNode};
use kodept_core::structure::span::CodeHolder;

use crate::graph::SubSyntaxTree;
use crate::traits::PopulateTree;
use crate::{node, node_sub_enum, BodyFnDecl, TyName, TyParam};

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
    type Root = StructDecl;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let node =
            StructDecl::uninit(context.get_chunk_located(&self.id).to_string()).with_rlt(self);
        SubSyntaxTree::new(node)
            .with_children_from(self.body.iter().flat_map(|x| x.inner.as_ref()), context)
            .with_children_from(
                self.parameters.iter().flat_map(|x| x.inner.as_ref()),
                context,
            )
    }
}

impl PopulateTree for Enum {
    type Root = EnumDecl;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        let (kind, name, rest) = match self {
            Enum::Stack { id, contents, .. } => (EnumKind::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumKind::Heap, id, contents),
        };
        let node =
            EnumDecl::uninit(kind, context.get_chunk_located(name).to_string()).with_rlt(self);
        SubSyntaxTree::new(node)
            .with_children_from(rest.iter().flat_map(|it| it.inner.as_ref()), context)
    }
}

impl PopulateTree for TopLevelNode {
    type Root = TopLevel;

    fn convert(&self, context: &mut impl CodeHolder) -> SubSyntaxTree<Self::Root> {
        match self {
            TopLevelNode::Enum(x) => x.convert(context).cast(),
            TopLevelNode::Struct(x) => x.convert(context).cast(),
            TopLevelNode::BodiedFunction(x) => x.convert(context).cast()
        }
    }
}
