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
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct StructDecl {
        pub name: String,;
        pub parameters: Vec<TyParam>,
        pub contents: Vec<BodyFnDecl>,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct EnumDecl {
        pub kind: EnumKind,
        pub name: String,;
        pub contents: Vec<TyName>,
    }
}

impl<'a> PopulateTree<'a> for &'a Struct {
    type Root = StructDecl;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let node =
            StructDecl::uninit(context.get_chunk_located(&self.id).to_string()).with_rlt(self);
        SubSyntaxTree::new(node)
            .maybe_with_children_from(self.body.as_ref().map(|x| x.inner.as_ref()), context)
            .maybe_with_children_from(self.parameters.as_ref().map(|x| x.inner.as_ref()), context)
    }
}

impl<'a> PopulateTree<'a> for &'a Enum {
    type Root = EnumDecl;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let (kind, name, rest) = match self {
            Enum::Stack { id, contents, .. } => (EnumKind::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumKind::Heap, id, contents),
        };
        let node =
            EnumDecl::uninit(kind, context.get_chunk_located(name).to_string()).with_rlt(self);
        SubSyntaxTree::new(node)
            .maybe_with_children_from(rest.as_ref().map(|it| it.inner.as_ref()), context)
    }
}

impl<'a> PopulateTree<'a> for &'a TopLevelNode {
    type Root = TopLevel;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            TopLevelNode::Enum(x) => x.convert(context).cast(),
            TopLevelNode::Struct(x) => x.convert(context).cast(),
            TopLevelNode::BodiedFunction(x) => x.convert(context).cast(),
        }
    }
}
