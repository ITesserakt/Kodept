use crate::constants::Const;
use crate::function::Func;
use crate::top_level::{EnumDecl, StructDecl};
use crate::utils::const_disjoint;
use crate::Unit;
use kodept_ast::{derive_node, relation};
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::children::ChildrenDisjoint;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_rlt::prelude as rlt;
use std::fmt::Debug;
use bevy_ecs::prelude::Component;
use kodept_ast::arity::Plural;

#[derive(Debug, PartialEq, Component)]
pub struct FileDecl;

#[derive(Debug, PartialEq, Component)]
pub enum ModDecl {
    Global,
    Ordinary,
}

derive_node!(FileDecl);
relation!(FileDecl => children ModDecl);

derive_node!(ModDecl {
    properties = [require Name,]
});
relation!(ModDecl => children Const);

impl FromSyntax for FileDecl {
    type Syntax = rlt::File;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, FileDecl)
            .with_children(source, pool, |scope| scope.many(node.0.as_ref()))
    }
}

impl FromSyntax for ModDecl {
    type Syntax = rlt::Module;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        let (kind, id, rest) = match node {
            rlt::Module::Global { id, rest, .. } => (ModDecl::Global, id, rest.as_ref()),
            rlt::Module::Ordinary { id, rest, .. } => (ModDecl::Ordinary, id, rest.as_ref()),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(pool, kind)
            .with_property(Name(name))
            .with_children(source, pool, |scope| scope.choose(Unit, rest))
    }
}

impl Choose<rlt::TopLevelNode, ModDecl, ()> for Unit {
    type Arity = Plural;
    
    #[inline(always)]
    fn branch<S: CodeHolder>(node: &rlt::TopLevelNode) -> ChildrenDisjoint<ModDecl, S, Self::Arity, ()> {
        match node {
            rlt::TopLevelNode::Enum(x) => const_disjoint::<EnumDecl, _, _, _>(x, |node, source: S| {
                source.get_chunk_located(match node {
                    rlt::Enum::Stack { id, .. } => id,
                    rlt::Enum::Heap { id, .. } => id,
                })
            }),
            rlt::TopLevelNode::Struct(x) => {
                const_disjoint::<StructDecl, _, _, _>(x, |node, source: S| {
                    source.get_chunk_located(&node.id)
                })
            }
            rlt::TopLevelNode::BodiedFunction(x) => {
                const_disjoint::<Func, _, _, _>(x, |node, source: S| {
                    source.get_chunk_located(&node.id)
                })
            }
        }
    }
}
