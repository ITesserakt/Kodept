use crate::impl_identifiable;
use crate::node_id::NodeId;
use crate::top_level::TopLevel;
use crate::traits::{IdProducer, Instantiable, IntoAst, Linker};
use kodept_core::structure::rlt::{File, Module};
use kodept_core::structure::span::CodeHolder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use visita::{node_group, Node, Visit, Visitor};

/// Abstract syntax tree
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct AST(pub FileDeclaration);

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "size-of", derive(SizeOf))]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct FileDeclaration {
    pub modules: Vec<ModuleDeclaration>,
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
    pub items: Vec<TopLevel>,
    id: NodeId<ModuleDeclaration>,
}

node_group! {
    family: FileDeclaration,
    nodes: [FileDeclaration,ModuleDeclaration]
}

node_group! {
    family: ModuleDeclaration,
    nodes: [ModuleDeclaration, TopLevel]
}

impl_identifiable! {
    FileDeclaration,
    ModuleDeclaration
}

impl IntoAst for File {
    type Output = FileDeclaration;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let node = FileDeclaration {
            modules: self.0.iter().map(|it| it.construct(context)).collect(),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl IntoAst for Module {
    type Output = ModuleDeclaration;

    fn construct<'x, P: IdProducer + Linker<'x> + CodeHolder>(
        &'x self,
        context: &mut P,
    ) -> Self::Output {
        let (kind, name, rest) = match self {
            Module::Global { id, rest, .. } => (ModuleKind::Global, id, rest),
            Module::Ordinary { id, rest, .. } => (ModuleKind::Ordinary, id, rest),
        };

        let node = ModuleDeclaration {
            kind,
            name: context.get_chunk_located(name).to_string(),
            items: rest.iter().map(|it| it.construct(context)).collect(),
            id: context.next_id(),
        };
        context.link(node, self)
    }
}

impl Instantiable for FileDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            modules: self
                .modules
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl Instantiable for ModuleDeclaration {
    fn new_instance<'c, P: IdProducer + Linker<'c>>(&'c self, context: &mut P) -> Self {
        let node = Self {
            kind: self.kind.clone(),
            name: self.name.clone(),
            items: self
                .items
                .iter()
                .map(|it| it.new_instance(context))
                .collect(),
            id: context.next_id(),
        };
        context.link_existing(node, self)
    }
}

impl AST {
    pub fn apply_visitor<V>(&mut self, mut visitor: V) -> V::Output
    where
        V: for<'a> Visitor<FileDeclaration, Data<'a> = ()>
            + Visit<FileDeclaration, FileDeclaration>,
    {
        self.0.accept(&mut visitor, ())
    }
}

#[cfg(test)]
mod tests {
    use crate::ast_builder::ASTBuilder;
    use crate::file::{FileDeclaration, ModuleDeclaration};
    use kodept_core::code_point::CodePoint;
    use kodept_core::structure::rlt;
    use kodept_core::structure::span::CodeHolder;
    use std::borrow::Cow;
    use visita::{impl_visitor, Node};

    struct CountVisitor(usize);

    impl_visitor! {
        CountVisitor,
        family: FileDeclaration,
        output: (),
        meta: (),
        [
            FileDeclaration => |this, node, meta| {
                node.modules.iter_mut().for_each(|it| it.accept(this, meta));
            },
            ModuleDeclaration => |this, node, meta| {
                this.0 += 1;
            }
        ]
    }

    struct EmptyCode;

    impl CodeHolder for EmptyCode {
        fn get_chunk(&self, _at: CodePoint) -> Cow<str> {
            "".into()
        }
    }

    #[test]
    fn test_empty_ast() {
        let rlt = rlt::File(Box::from([]));
        let mut visitor = CountVisitor(0);
        let (mut ast, _) = ASTBuilder::default().recursive_build(&rlt, &EmptyCode);

        ast.accept(&mut visitor, ());
        assert_eq!(visitor.0, 0);
    }
}
