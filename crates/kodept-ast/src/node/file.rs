#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

use kodept_core::structure::rlt::{File, Module};
use kodept_core::structure::span::CodeHolder;

use crate::{node, TopLevel};
use crate::graph::{NodeId, SyntaxTreeBuilder};
use crate::traits::Linker;
use crate::traits::PopulateTree;

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ModuleKind {
    Global,
    Ordinary,
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct FileDeclaration {
        ;
        pub modules: Vec<ModuleDeclaration>,
    }
}

node! {
    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "size-of", derive(SizeOf))]
    #[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
    pub struct ModuleDeclaration {
        pub kind: ModuleKind,
        pub name: String,;
        pub contents: Vec<TopLevel>,
    }
}

impl PopulateTree for File {
    type Output = FileDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        builder
            .add_node(FileDeclaration {
                id: NodeId::default(),
            })
            .with_children_from(self.0.iter(), context)
            .with_rlt(context, self)
            .id()
    }
}

impl PopulateTree for Module {
    type Output = ModuleDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTreeBuilder,
        context: &mut (impl Linker<'a> + CodeHolder),
    ) -> NodeId<Self::Output> {
        let (kind, name, rest) = match self {
            Module::Global { id, rest, .. } => (ModuleKind::Global, id, rest),
            Module::Ordinary { id, rest, .. } => (ModuleKind::Ordinary, id, rest),
        };
        let node = ModuleDeclaration {
            kind,
            name: context.get_chunk_located(name).to_string(),
            id: NodeId::default(),
        };
        builder
            .add_node(node)
            .with_children_from(rest.iter(), context)
            .with_rlt(context, self)
            .id()
    }
}