use crate::constants::Const;
use crate::function::Func;
use crate::top_level::{EnumDecl, StructDecl};
use crate::utils::const_disjoint;
use crate::Unit;
use kodept_ast::derive_node;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::tags::NoTag;
use kodept_ast::properties::{Name, RequireProperty};
use kodept_ast::syntax_tree::children::ChildrenDisjoint;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_rlt::prelude::{Enum, File, Module, TopLevelNode};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Component)]
pub struct FileDecl;

#[derive(Debug, PartialEq, Component)]
pub enum ModDecl {
    Global,
    Ordinary,
}

derive_node!(FileDecl {
    relations = [children ModDecl,],
    properties = []
});
derive_node!(ModDecl {
    relations = [
        children Const,
    ],
    properties = []
});

impl FromSyntax for FileDecl {
    type Syntax = File;

    fn from_syntax(node: &File, source_code: impl CodeHolder, builder: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(builder, FileDecl)
            .with_children(source_code, builder, |scope| scope.many(node.0.as_ref()))
    }
}

impl FromSyntax for ModDecl {
    type Syntax = Module;

    fn from_syntax(node: &Module, source_code: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let (kind, id, rest) = match node {
            Module::Global { id, rest, .. } => (ModDecl::Global, id, rest.as_ref()),
            Module::Ordinary { id, rest, .. } => (ModDecl::Ordinary, id, rest.as_ref()),
        };
        let name = source_code.get_chunk_located(id);
        ASTBuilder::new(pool, kind)
            .with_property(Name(name))
            .with_children(source_code, pool, |scope| scope.choose(Unit, rest))
    }
}

impl Choose<TopLevelNode, ModDecl, NoTag> for Unit {
    #[inline(always)]
    fn branch<S: CodeHolder>(node: &TopLevelNode) -> ChildrenDisjoint<ModDecl, S, NoTag> {
        match node {
            TopLevelNode::Enum(x) => const_disjoint::<EnumDecl, _, _, _>(x, |node, source: S| {
                source.get_chunk_located(match node {
                    Enum::Stack { id, .. } => id,
                    Enum::Heap { id, .. } => id,
                })
            }),
            TopLevelNode::Struct(x) => {
                const_disjoint::<StructDecl, _, _, _>(x, |node, source: S| {
                    source.get_chunk_located(&node.id)
                })
            }
            TopLevelNode::BodiedFunction(x) => {
                const_disjoint::<Func, _, _, _>(x, |node, source: S| {
                    source.get_chunk_located(&node.id)
                })
            }
        }
    }
}

impl RequireProperty<Name> for ModDecl {}
