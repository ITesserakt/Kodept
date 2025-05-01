use crate::report::Reporter;
use crate::scope::storage::ScopeBuilder;
use crate::wrapper::InteractionWrapper;
use crate::{done, Interaction};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Changed, ChildOf, Children, IntoScheduleConfigs, Or, Query, With};
use kodept_ast::define_union;
use kodept_ast::prelude::IntoEnum;
use kodept_ast::properties::{Node, SourceSpan};
use kodept_ast::query::AnyNodeQuery;
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::{FileDecl, ModDecl};
use kodept_ast_nodes::function::Func;
use kodept_ast_nodes::top_level::{EnumDecl, StructDecl};
use kodept_report::message::Severity;
use kodept_report::prelude::{Diagnostic, Label};
use std::convert::Infallible;

define_union!(enum ScopeUnion[ScopeUnionItem, ScopeUnionFilter] {
    FileDecl | ModDecl | StructDecl | EnumDecl | Func | Lambda | Exprs | IfExpr
});

#[derive(Debug)]
pub struct ScopeBuildingPass;

impl Interaction for ScopeBuildingPass {
    type Error = Infallible;

    fn interaction() -> InteractionWrapper<Self::Error> {
        type NodeFilter = (With<Node>, Or<(Changed<ChildOf>, Changed<Children>)>);

        let config = InteractionWrapper::wrap(Self::system)
            .unwrap()
            .run_if(|query: Query<(), NodeFilter>| !query.is_empty());
        InteractionWrapper::from_configs(config)
    }
}

impl ScopeBuildingPass {
    fn divide_by_scopes(entity: ScopeUnion, builder: &mut ScopeBuilder) -> Entity {
        let (name, is_anonymous, opaque) = match &*entity {
            ScopeUnionItem::FileDecl(_) => (None, false, false), 
            ScopeUnionItem::ModDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::StructDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::EnumDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::Func(x) => (Some(x.name().clone()), true, true),
            ScopeUnionItem::Lambda(_) => (None, true, false),
            ScopeUnionItem::Exprs(_) => (None, true, false),
            ScopeUnionItem::IfExpr(_) => (None, true, false),
        };
        builder.allocate_scope(entity.id, name, is_anonymous, opaque)
    }

    fn system(
        nodes: AnyNodeQuery,
        spans: Query<&SourceSpan>,
        reporter: Reporter,
        mut builder: ScopeBuilder
    ) -> crate::Result<Infallible> {
        for (parent_id, node_id) in nodes.iter_top_down() {
            let node = nodes.get(node_id).unwrap();
            let parent_scope = parent_id.and_then(|it| builder.get_enclosing_scope(it));
            
            if let Some(node_enum) = node.into_enum() {
                let this_scope_id = Self::divide_by_scopes(node_enum, &mut builder);
                parent_scope.map(|it| builder.link_scopes(this_scope_id, it));
            } else if let Some(parent) = parent_scope {
                builder.set_enclosing_scope(node_id, parent);
            } else {
                reporter.report_ad_hoc(|| {
                    Diagnostic::new(Severity::Bug)
                        .with_label(Label::primary("", spans.get(parent_scope.unwrap()).unwrap().0))
                        .with_message("No scope associated with this element")
                        .with_note(format!("Entity {}", node.id()))
                })
            }
        }

        done()
    }
}
