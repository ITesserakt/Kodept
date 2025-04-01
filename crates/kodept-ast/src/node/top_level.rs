#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use kodept_rlt::prelude::{Enum, Struct, TopLevelNode};

use crate::graph::SubSyntaxTree;
use crate::traits::{CodeHolder, PopulateTree};
use crate::{node, node_sub_enum, BodyFnDecl, ModDecl, Str, TyName, TyParam};

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
        pub name: Str,;
        pub parameters: Vec<TyParam>,
        pub contents: Vec<BodyFnDecl>,;
        parent is [ModDecl]
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct EnumDecl {
        pub kind: EnumKind,
        pub name: Str,;
        pub contents: Vec<TyName>,;
        parent is [ModDecl]
    }
}

impl<'a> PopulateTree<'a> for &'a Struct {
    type Root = StructDecl;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let node =
            StructDecl::uninit(context.get_chunk_located(&self.id)).with_rlt(self);
        SubSyntaxTree::new(node)
            .maybe_with_children_from(self.body.as_ref().map(|x| x.inner.as_ref()), context)
            .maybe_with_children_from(self.parameters.as_ref().map(|x| x.inner.as_ref()), context)
    }
}

impl<'a> PopulateTree<'a> for &'a Enum {
    type Root = EnumDecl;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        let (kind, name, rest) = match self {
            Enum::Stack { id, contents, .. } => (EnumKind::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumKind::Heap, id, contents),
        };
        let node =
            EnumDecl::uninit(kind, context.get_chunk_located(name)).with_rlt(self);
        SubSyntaxTree::new(node)
            .maybe_with_children_from(rest.as_ref().map(|it| it.inner.as_ref()), context)
    }
}

impl<'a> PopulateTree<'a> for &'a TopLevelNode {
    type Root = TopLevel;

    fn convert(self, context: impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
        match self {
            TopLevelNode::Enum(x) => x.convert(context).cast(),
            TopLevelNode::Struct(x) => x.convert(context).cast(),
            TopLevelNode::BodiedFunction(x) => x.convert(context).cast(),
        }
    }
}
