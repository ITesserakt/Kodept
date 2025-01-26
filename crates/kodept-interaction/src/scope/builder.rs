use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::ScopeMapping;
use crate::wrapper::InteractionWrapper;
use crate::{done, Interaction};
use bevy_ecs::change_detection::{Res, ResMut};
use bevy_ecs::entity::{Entities, Entity};
use bevy_ecs::prelude::{Changed, Commands, Or, Query, Single, With};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_hierarchy::{BuildChildren, Children, HierarchyQueryExt, Parent};
use kodept_ast::define_union;
use kodept_ast::prelude::{AnyNodeRef, IntoEnum};
use kodept_ast::properties::{Node, Root};
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
use kodept_ast_nodes::types::{Params, TyParams};

define_union!(enum ScopeUnion[ScopeUnionItem, ScopeUnionFilter] {
    ModDecl | StructDecl | EnumDecl | Func | Params | TyParams | Lambda | Exprs | IfExpr
});

#[derive(Debug)]
pub struct ScopeBuilder;

impl Interaction for ScopeBuilder {
    type Error = Infallible;

    fn interaction() -> InteractionWrapper<Self::Error> {
        type NodeFilter = (With<Node>, Or<(Changed<Parent>, Changed<Children>)>);

        let config = InteractionWrapper::wrap(Self::scope_builder_interaction)
            .unwrap()
            .run_if(|query: Query<(), NodeFilter>| !query.is_empty());
        InteractionWrapper::from_configs(config)
    }
}

impl ScopeBuilder {
    fn divide_by_scopes(entity: ScopeUnion, spawner: &Entities, commands: &mut Commands) -> Entity {
        let (name, is_anonymous) = match &*entity {
            ScopeUnionItem::ModDecl(x) => (Some(x.name().clone()), false),
            ScopeUnionItem::StructDecl(x) => (Some(x.name().clone()), false),
            ScopeUnionItem::EnumDecl(x) => (Some(x.name().clone()), false),
            ScopeUnionItem::Func(x) => (Some(x.name().clone()), true),
            ScopeUnionItem::Lambda(_) => (None, true),
            ScopeUnionItem::Exprs(_) => (None, true),
            ScopeUnionItem::IfExpr(_) => (None, true),
            ScopeUnionItem::Params(_) => (None, false),
            ScopeUnionItem::TyParams(_) => (None, false),
        };
        let scope_id = spawner.reserve_entity();
        if let Some(name) = name {
            commands
                .entity(scope_id)
                .insert((name, Scope::new(entity.id, is_anonymous)));
        } else {
            commands
                .entity(scope_id)
                .insert(Scope::new(entity.id, is_anonymous));
        }
        scope_id
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
        scopes: Option<ResMut<ScopeMapping>>,
    ) -> crate::Result<Infallible> {
        let mut enclosing_scopes = scopes
            .map(|mut it| std::mem::take(&mut it.enclosing_scopes_mapping))
            .unwrap_or_default();
        let root_scope = spawner.reserve_entity();
        enclosing_scopes.insert(*root, root_scope);

        for entity in children.iter_descendants_depth_first(*root) {
            let node = query.get(entity).unwrap();
            let parent_scope_id = parents
                .iter_ancestors(entity)
                .find_map(|it| enclosing_scopes.get(&it))
                .copied();

            if let Some(scope_node) = node.into_enum() {
                let scope_id = Self::divide_by_scopes(scope_node, spawner, &mut commands);
                if let Some(parent) = parent_scope_id {
                    commands.entity(scope_id).set_parent(parent);
                }
                enclosing_scopes.insert(entity, scope_id);
            } else if let Some(parent) = parent_scope_id {
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

        commands.entity(root_scope).insert(Scope {
            start_from: *root,
            is_anonymous: false,
        });

        commands.insert_resource(ScopeMapping {
            enclosing_scopes_mapping: enclosing_scopes,
        });

        done()
    }
}
