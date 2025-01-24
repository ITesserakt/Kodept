use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::{done, Interacted, Interaction};
use bevy_ecs::entity::{Entities, EntityHashMap};
use bevy_ecs::prelude::{Commands, Entity, IntoSystem, Query, Res, Resource, Single, With};
use bevy_hierarchy::{BuildChildren, Children, HierarchyQueryExt, Parent};
use kodept_ast::prelude::{AnyNodeRef, AnyNodeRefItem};
use kodept_ast::properties::{Name, Node, Root};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::ModDecl;
use kodept_ast_nodes::function::Func;
use kodept_ast_nodes::top_level::{EnumDecl, StructDecl};
use kodept_core::structure::Located;
use kodept_report::error::report::{Label, Severity};
use kodept_report::error::Diagnostic;
use std::convert::Infallible;

mod storage;
mod symbol;

#[derive(Debug)]
pub struct ScopeBuilder;

#[derive(Debug, Resource)]
pub struct ScopeMapping {
    /// Mapping from entity to its enclosing scope entity
    enclosing_scopes_mapping: EntityHashMap<Entity>,
}

impl Interaction for ScopeBuilder {
    type Error = Infallible;

    fn interaction() -> impl IntoSystem<(), Interacted<Self::Error>, ()> {
        IntoSystem::into_system(Self::scope_builder_interaction)
    }
}

impl ScopeBuilder {
    fn non_anonymous_scope(entity: AnyNodeRefItem) -> Option<Name> {
        if let Some(node) = entity.cast::<ModDecl>() {
            Some(node.property::<Name>().clone())
        } else if let Some(node) = entity.cast::<StructDecl>() {
            Some(node.property::<Name>().clone())
        } else if let Some(node) = entity.cast::<EnumDecl>() {
            Some(node.property::<Name>().clone())
        } else {
            None
        }
    }
    
    fn anonymous_scope(entity: AnyNodeRefItem) -> Option<Name> {
        if let Some(node) = entity.cast::<Func>() {
            Some(node.property::<Name>().clone())
        } else {
            None
        }
    }
    
    fn unnamed_scope(entity: AnyNodeRefItem) -> bool {
        if let Some(_) = entity.cast::<Lambda>() {
            true
        } else if let Some(_) = entity.cast::<Exprs>() {
            true
        } else if let Some(_) = entity.cast::<IfExpr>() {
            true
        } else {
            false
        }
    }

    fn divide_by_scopes(
        entity: AnyNodeRefItem,
        spawner: &Entities,
        commands: &mut Commands,
    ) -> Option<Entity> {
        if let Some(name) = Self::non_anonymous_scope(entity) {
            let scope_id = spawner.reserve_entity();
            commands.entity(scope_id).insert((
                name,
                Scope {
                    start_from: entity.id(),
                    is_anonymous: false,
                },
            ));
            Some(scope_id)
        } else if let Some(name) = Self::anonymous_scope(entity) {
            let scope_id = spawner.reserve_entity();
            commands.entity(scope_id).insert((
                name,
                Scope {
                    start_from: entity.id(),
                    is_anonymous: true,
                }
            ));
            Some(scope_id)
        } else if Self::unnamed_scope(entity) {
            let scope_id = spawner.reserve_entity();
            commands.entity(scope_id).insert(Scope {
                start_from: entity.id(),
                is_anonymous: true,
            });
            Some(scope_id)
        } else {
            None
        }
    }

    fn scope_builder_interaction(
        root: Single<Entity, With<Root>>,
        query: Query<AnyNodeRef, With<Node>>,
        children: Query<&Children, With<Node>>,
        parents: Query<&Parent, With<Node>>,
        spawner: &Entities,
        mut commands: Commands,
        reporter: Reporter,
        syntax: Res<SyntaxResolver>,
    ) -> Interacted<Infallible> {
        let mut enclosing_scopes = EntityHashMap::default();

        for entity in children.iter_descendants(*root) {
            let node = query.get(entity).unwrap();
            let parent_scope_id = parents
                .iter_ancestors(entity)
                .find_map(|it| enclosing_scopes.get(&it))
                .copied();

            if let Some(scope_id) = Self::divide_by_scopes(node, spawner, &mut commands) {
                if let Some(parent) = parent_scope_id {
                    commands.entity(scope_id).set_parent(parent);
                }
                enclosing_scopes.insert(entity, scope_id);
            } else {
                if let Some(parent) = parent_scope_id {
                    enclosing_scopes.insert(entity, parent);
                } else {
                    reporter.report_ad_hoc(|| {
                        let point = syntax.get_unknown(entity).unwrap().location();
                        Diagnostic::new(Severity::Bug)
                            .with_label(Label::primary("", point))
                            .with_message("No scope associated with this element")
                            .with_note(format!("Entity {}", entity))
                    });
                }
            }
        }

        commands.insert_resource(ScopeMapping {
            enclosing_scopes_mapping: enclosing_scopes,
        });

        done()
    }
}
