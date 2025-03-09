use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::ScopeMapping;
use crate::wrapper::InteractionWrapper;
use crate::{done, Interaction};
use bevy_ecs::change_detection::{Res, ResMut};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, Commands, Or, Query, With};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_hierarchy::{BuildChildren, Children, Parent};
use kodept_ast::define_union;
use kodept_ast::prelude::IntoEnum;
use kodept_ast::properties::Node;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::prelude::ASTQuery;
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::ModDecl;
use kodept_ast_nodes::function::Func;
use kodept_ast_nodes::top_level::{EnumDecl, StructDecl};
use kodept_core::structure::Located;
use kodept_report::message::{Diagnostic, Label, Severity};
use std::convert::Infallible;

define_union!(enum ScopeUnion[ScopeUnionItem, ScopeUnionFilter] {
    ModDecl | StructDecl | EnumDecl | Func | Lambda | Exprs | IfExpr
});

#[derive(Debug)]
pub struct ScopeBuilder;

impl Interaction for ScopeBuilder {
    type Error = Infallible;

    fn interaction() -> InteractionWrapper<Self::Error> {
        type NodeFilter = (With<Node>, Or<(Changed<Parent>, Changed<Children>)>);

        let config = InteractionWrapper::wrap(Self::system)
            .unwrap()
            .run_if(|query: Query<(), NodeFilter>| !query.is_empty());
        InteractionWrapper::from_configs(config)
    }
}

impl ScopeBuilder {
    fn divide_by_scopes(entity: ScopeUnion, commands: &mut Commands) -> Entity {
        let (name, is_anonymous, opaque) = match &*entity {
            ScopeUnionItem::ModDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::StructDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::EnumDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::Func(x) => (Some(x.name().clone()), true, true),
            ScopeUnionItem::Lambda(_) => (None, true, false),
            ScopeUnionItem::Exprs(_) => (None, true, false),
            ScopeUnionItem::IfExpr(_) => (None, true, false),
        };
        if let Some(name) = name {
            commands
                .spawn((name, Scope::new(entity.id, is_anonymous).opaque(opaque)))
                .id()
        } else {
            commands
                .spawn(Scope::new(entity.id, is_anonymous).opaque(opaque))
                .id()
        }
    }

    fn system(
        query: ASTQuery,
        mut commands: Commands,
        reporter: Reporter,
        syntax: Res<SyntaxResolver>,
        scopes: Option<ResMut<ScopeMapping>>,
    ) -> crate::Result<Infallible> {
        let mut enclosing_scopes = scopes
            .map(|mut it| std::mem::take(&mut it.enclosing_scopes_mapping))
            .unwrap_or_default();

        let root_scope = commands.spawn(Scope::new(query.root(), false)).id();
        enclosing_scopes.insert(*query.root(), root_scope);
        
        for node in query.iter_descendants() {
            let parent_scope_id = query
                .iter_ancestors(node.id())
                .find_map(|it| enclosing_scopes.get(&*it.id()))
                .copied();

            if let Some(scope_node) = node.into_enum() {
                let scope_id = Self::divide_by_scopes(scope_node, &mut commands);
                if let Some(parent) = parent_scope_id {
                    commands.entity(scope_id).set_parent(parent);
                }
                enclosing_scopes.insert(*node.id(), scope_id);
            } else if let Some(parent) = parent_scope_id {
                enclosing_scopes.insert(node.id().into(), parent);
            } else {
                reporter.report_ad_hoc(|| {
                    let point = syntax.try_get_unknown(node.id()).unwrap().location();
                    Diagnostic::new(Severity::Bug)
                        .with_label(Label::primary("", point))
                        .with_message("No scope associated with this element")
                        .with_note(format!("Entity {}", node.id()))
                });
            }
        }

        commands.insert_resource(ScopeMapping::new(enclosing_scopes, query.root()));

        done()
    }
}
