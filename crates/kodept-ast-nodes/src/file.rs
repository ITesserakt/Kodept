use crate::consts::Const;
use bevy_ecs::component::Component;
use bevy_ecs::relationship::Relationship;
use bevy_ecs::system::Commands;
use kodept_ast::derive_node;
use kodept_ast::experimental::FromSyntax;
use kodept_ast::experimental::{AstBuilder, SpawnContext};
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::relation;
use kodept_ast::syntax_tree::experimental::SpawnedIn;
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude as rlt;

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

impl FileDecl {
    pub fn from_syntax(
        node: &rlt::File,
        source: impl CodeHolder,
        commands: Commands,
    ) -> Result<NodeId<Self>, crate::Error> {
        SpawnContext::top_level(node, commands, source)
    }
}

impl FromSyntax<rlt::File> for FileDecl {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &rlt::File,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        AstBuilder::new(FileDecl)
            .with_property(SourceSpan(node.bounds()))
            .spawn_in(spawner)
            .with_children(node.0.as_ref(), source)
            .map(|it| it.finish())
    }
}

impl FromSyntax<rlt::Module> for ModDecl {
    type Error = crate::Error;

    fn from_syntax<R: Relationship>(
        node: &rlt::Module,
        spawner: SpawnContext<R>,
        source: impl CodeHolder,
    ) -> Result<NodeId<Self>, Self::Error> {
        let (value, name, rest) = match node {
            rlt::Module::Global { id, rest, .. } => {
                (ModDecl::Global, source.get_chunk_located(id), rest)
            }
            rlt::Module::Ordinary { id, rest, .. } => {
                (ModDecl::Ordinary, source.get_chunk_located(id), rest)
            }
        };
        AstBuilder::new(value)
            .with_property(SourceSpan(node.bounds()))
            .with_property(Name::new(name))
            .spawn_in(spawner)
            .with_children(rest.as_ref(), source)
            .map(|it| it.finish())
    }
}
