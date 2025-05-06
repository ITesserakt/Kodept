use crate::scope::storage::Scope;
use crate::scope::Scoped;
use crate::wrapper::InteractionExt;
use crate::{done, fail, Ctx, Interaction};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{any_match_filter, Commands, IntoScheduleConfigs, Query, With, Without};
use bevy_ecs::relationship::Relationship;
use kodept_ast::define_union;
use kodept_ast::prelude::{AnyNodeRef, ChildOf, IntoEnum};
use kodept_ast::properties::{Node, SourceSpan};
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::{FileDecl, ModDecl};
use kodept_ast_nodes::function::FuncBody;
use kodept_ast_nodes::top_level::{EnumDecl, StructDecl};
use kodept_report::prelude::{
    Diagnostic, IntoSpannedReportMessage, Label, MessageBehaviour, Severity,
};

define_union!(enum ScopeUnion[ScopeUnionItem, ScopeUnionFilter] {
    FileDecl | ModDecl | StructDecl | EnumDecl | FuncBody | Lambda | Exprs | IfExpr
});

#[derive(Debug)]
pub struct ScopeBuildingPass;

pub struct CannotLinkError(Option<SourceSpan>);

impl IntoSpannedReportMessage for CannotLinkError {
    type Message = Diagnostic;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Critical bug in compiler, cannot proceed")
    }

    fn into_message(self) -> Self::Message {
        let mut diag = Diagnostic::new(Severity::Bug)
            .with_message("Cannot create new scope or link with any other")
            .with_note("Possible out-of-tree nodes?");
        if let Some(last) = self.0 {
            diag = diag.with_label(Label::primary("unprocessed node", last))
        }
        diag
    }
}

impl Interaction for ScopeBuildingPass {
    type Error = CannotLinkError;

    fn install(ctx: &mut Ctx) {
        ctx.register(
            Self::wrap_system(Self::system)
                .run_if(any_match_filter::<(With<Node>, Without<Scoped>)>),
        )
    }
}

impl ScopeBuildingPass {
    fn divide_by_scopes(entity: ScopeUnion, commands: &mut Commands) -> Entity {
        let (name, is_anonymous, is_opaque) = match &*entity {
            ScopeUnionItem::FileDecl(_) => (None, false, false),
            ScopeUnionItem::ModDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::StructDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::EnumDecl(x) => (Some(x.name().clone()), false, false),
            ScopeUnionItem::FuncBody(_) => (None, true, true),
            ScopeUnionItem::Lambda(_) => (None, true, false),
            ScopeUnionItem::Exprs(_) => (None, true, false),
            ScopeUnionItem::IfExpr(_) => (None, true, false),
        };

        let mut scope_entity = if let Some(name) = name {
            commands.spawn((Scope::new(entity.id, is_anonymous).opaque(is_opaque), name))
        } else {
            commands.spawn(Scope::new(entity.id, is_anonymous).opaque(is_opaque))
        };

        scope_entity
            .add_one_related::<Scoped>(entity.id.entity())
            .id()
    }

    /// If the current node is a start of scope, spawn new scope.
    ///
    /// Otherwise, propagate `scoped` component from parent if it has one
    fn system(
        unscoped: Query<(AnyNodeRef, Option<&ChildOf>), (Without<Scoped>, With<Node>)>,
        scoped: Query<&Scoped>,
        mut commands: Commands,
    ) -> crate::Result<CannotLinkError> {
        let mut amount_of_processed = 0;
        let mut last_unprocessed = None;

        unscoped.iter().for_each(|(node, parent)| {
            if let Some(node_enum) = node.into_enum() {
                Self::divide_by_scopes(node_enum, &mut commands);
                amount_of_processed += 1;
            } else if let Some(scope) = parent.and_then(|it| scoped.get(it.get()).ok()) {
                commands
                    .entity(scope.0)
                    .add_one_related::<Scoped>(node.id().entity());
                amount_of_processed += 1;
            } else {
                // We should repeat the whole process to get more `scoped`...
                // If there are no processed nodes, then it's a bug
                last_unprocessed = Some(node.span());
            }
        });

        if amount_of_processed == 0 {
            fail(CannotLinkError(last_unprocessed))?;
        }

        done()
    }
}
