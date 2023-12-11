use crate::graph::traits::PopulateTree;
use crate::graph::SyntaxTree;
use crate::node_id::NodeId;
use crate::traits::Linker;
use crate::{impl_identifiable_2, with_children, TopLevel};
use derive_more::Constructor;
use kodept_core::structure::rlt::{File, Module};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

/// Abstract syntax tree
#[derive(Debug, Constructor)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AST(pub(crate) SyntaxTree);

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct FileDeclaration {
    id: NodeId<FileDeclaration>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum ModuleKind {
    Global,
    Ordinary,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ModuleDeclaration {
    pub kind: ModuleKind,
    pub name: String,
    id: NodeId<ModuleDeclaration>,
}

impl_identifiable_2! { FileDeclaration, ModuleDeclaration }

with_children!(FileDeclaration => {
    pub modules: Vec<ModuleDeclaration>
});

with_children!(ModuleDeclaration => {
    pub contents: Vec<TopLevel>
});

impl PopulateTree for File {
    type Output = FileDeclaration;

    fn convert<'a>(
        &'a self,
        builder: &mut SyntaxTree,
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
        builder: &mut SyntaxTree,
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
