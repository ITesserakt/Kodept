use crate::graph::SubSyntaxTree;
use crate::traits::PopulateTree;
use crate::{node, TopLevel};
use kodept_core::structure::rlt;
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use crate::interning::SharedStr;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ModuleKind {
    Global,
    Ordinary,
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct FileDecl {
        ;
        pub modules: Vec<ModDecl>,
    }
}

node! {
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ModDecl {
        pub kind: ModuleKind,
        pub name: SharedStr,;
        pub contents: Vec<TopLevel>,;
        parent is [FileDecl]
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::File {
    type Root = FileDecl;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        let node = FileDecl::uninit().with_rlt(self);
        SubSyntaxTree::new(node).with_children_from(self.0.as_ref(), context)
    }
}

impl<'a> PopulateTree<'a> for &'a rlt::Module {
    type Root = ModDecl;

    fn convert(self, context: impl CodeHolder<Str = SharedStr>) -> SubSyntaxTree<'a, Self::Root> {
        let (kind, name, rest) = match self {
            rlt::Module::Global { id, rest, .. } => {
                (ModuleKind::Global, context.get_chunk_located(id), rest)
            }
            rlt::Module::Ordinary { id, rest, .. } => {
                (ModuleKind::Ordinary, context.get_chunk_located(id), rest)
            }
        };
        let node = ModDecl::uninit(kind, name).with_rlt(self);
        SubSyntaxTree::new(node).with_children_from(rest.as_ref(), context)
    }
}
