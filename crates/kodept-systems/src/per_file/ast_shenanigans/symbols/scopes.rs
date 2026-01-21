use super::*;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QuerySingleError;
use bevy_ecs::relationship::Relationship;
use kodept_ast::properties::{Node, SourceSpan};

pub(super) fn spawn_scope<T: ASTNode>(
    params: ScopeSpawnParams,
) -> impl FnMut(Populated<(Entity, Option<&Name>), (Without<InScope>, With<T>)>, Commands) {
    move |query, mut commands| {
        for (id, item_name) in query {
            if let Some(name) = &params.override_name {
                commands
                    .spawn((Scope { starts_from: id }, name.clone()))
                    .add_one_related::<InScope>(id);
            } else if let Some(name) = item_name {
                commands
                    .spawn((Scope { starts_from: id }, name.clone()))
                    .add_one_related::<InScope>(id);
            } else {
                commands
                    .spawn((Scope { starts_from: id },))
                    .add_one_related::<InScope>(id);
            }
        }
    }
}

/// Sets scope of a node to be equal to the parent one if not exist
pub(super) fn propagate_scopes(
    query: Populated<(Entity, Option<&ChildOf>, &SourceSpan, &Node), Without<InScope>>,
    scopes: Query<&InScope>,
    mut commands: Commands,
) -> Result<(), CannotLinkError> {
    let mut any_processed = false;
    let mut last_unprocessed = None;
    for (id, parent, span, node) in query {
        let Some(scope) = parent.and_then(|it| scopes.get(it.get()).ok()) else {
            last_unprocessed = Some((span, &node.kind));
            continue;
        };
        // parent was scoped already
        any_processed = true;
        commands.entity(scope.0).add_one_related::<InScope>(id);
    }

    if let Some((span, kind)) = last_unprocessed
        && !any_processed
    {
        Err(CannotLinkError {
            node_location: *span,
            node_kind: kind.as_string(),
        })
    } else {
        Ok(())
    }
}

pub(super) fn link_scopes(
    _: On<ScopesBuiltEvent>,
    query: Populated<(Option<&ChildOf>, &InScope)>,
    scopes: Query<&Children, With<Scope>>,
    mut commands: Commands,
) {
    for (parent, scope) in query.iter() {
        let Some((_, parent_scope)) = parent.and_then(|it| query.get(it.0).ok()) else {
            continue;
        };
        if parent_scope == scope {
            continue;
        }
        let parent_scope_children = scopes.get(parent_scope.0).ok();
        if parent_scope_children.is_none_or(|it| !it.contains(&scope.0)) {
            commands.entity(parent_scope.0).add_child(scope.0);
        }
    }
    commands.trigger(ScopesLinkedEvent);
}

pub(super) fn ensure_one_root_scope(
    _: On<ScopesLinkedEvent>,
    query: Query<&Scope, Without<ChildOf>>,
    nodes: Query<(&SourceSpan, &Node)>,
) -> Result<(), MultipleRootScopes> {
    let Err(QuerySingleError::MultipleEntities(_)) = query.single() else {
        return Ok(());
    };

    Err(MultipleRootScopes(
        query
            .iter()
            .filter_map(|it| nodes.get(it.starts_from).ok())
            .map(|it| (*it.0, it.1.kind.as_string()))
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::per_file::ast_shenanigans::symbols::scopes::spawn_scope;
    use crate::per_file::ast_shenanigans::symbols::{ScopeSpawnParams, Scoping};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use kodept_ast::relationship::Nodes;
    use kodept_ast_nodes::v2::file::{FileDecl, ModDecl};

    #[test]
    fn test_scopes_spawn() {
        let mut world = World::new();

        world.spawn((
            FileDecl,
            related!(Nodes [
                ModDecl::Ordinary,
                ModDecl::Ordinary,
            ]),
        ));

        world
            .run_system_once(spawn_scope::<FileDecl>(ScopeSpawnParams::default()))
            .unwrap();
        world
            .run_system_once(spawn_scope::<ModDecl>(ScopeSpawnParams::default()))
            .unwrap();

        let mut query = world.query::<&Scoping>();
        let query = query.iter(&world);
        assert_eq!(query.len(), 3);
    }
}
