use bevy_ecs::entity::Entities;
use bevy_ecs::prelude::{Component, Entity, ParallelCommands, Populated, Without};
use bevy_hierarchy::BuildChildren;
use kodept::source_files::SourceView;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::span::CodeHolder;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;
use std::borrow::Cow;
use std::ops::Deref;

pub struct BuildASTPlugin;

#[derive(Debug, Component)]
pub struct Built;

impl Plugin for BuildASTPlugin {
    fn build(self, world: &mut Frontend) {
        world.add_systems(build_ast_system);
    }
}

fn build_ast_system(
    query: Populated<(Entity, &SyntaxResolver, &SourceView), Without<Built>>,
    entities: &Entities,
    commands: ParallelCommands,
) {
    query.par_iter().for_each(|(entity, it, source)| {
        let helper_fn = || {
            #[cfg(feature = "interning")]
            {
                let code_holder = kodept_interning::InterningCodeHolder::new(source.deref())
                    .map(|it| Cow::Borrowed(it.0));
                return AST::create_with_resolver::<FileDecl>(code_holder, it, entities);
            }
            #[cfg(not(feature = "interning"))]
            {
                let code_holder = source.deref().map(|it| Cow::Owned(it.to_string()));
                return AST::create_with_resolver::<FileDecl>(code_holder, it, entities);
            }
        };
        let (root, mut queue) = helper_fn();
        commands.command_scope(|mut commands| {
            commands.entity(entity).insert(Built).add_child(root);
            commands.append(&mut queue);
        });
    });
}
