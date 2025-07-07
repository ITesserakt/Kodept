use crate::phase::{CurrentPhase, Phase};
use crate::scope::storage::Scope;
use crate::scope::Scoped;
use crate::wrapper::InteractionExt;
use crate::{done, fail, Ctx, Interaction};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{
    Commands, IntoScheduleConfigs, Populated, Query, SystemSet, With, Without,
};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::system::ResMut;
use kodept_ast::define_union;
use kodept_ast::prelude::{AnyNodeRef, ChildOf, IntoEnum};
use kodept_ast::properties::{Node, SourceSpan};
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::{FileDecl, ModDecl};
use kodept_ast_nodes::function::FuncDecl;
use kodept_ast_nodes::top_level::{EnumDecl, StructDecl};
use kodept_report::prelude::{
    Diagnostic, IntoSpannedReportMessage, Label, MessageBehaviour, Severity,
};

define_union!(enum ScopeUnion[ScopeUnionItem, ScopeUnionFilter] {
    FileDecl | ModDecl | StructDecl | EnumDecl | FuncDecl | Lambda | Exprs | IfExpr
});

#[derive(Debug, SystemSet, Clone, Hash, Eq, PartialEq)]
pub struct ScopeBuildingPass;

pub struct CannotLinkError(SourceSpan);

impl IntoSpannedReportMessage for CannotLinkError {
    type Message = Diagnostic;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Critical bug in compiler, cannot proceed")
    }

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Bug)
            .with_message("Cannot create new scope or link with any other")
            .with_note("Possible out-of-tree nodes?")
            .with_label(Label::primary("unprocessed node", self.0))
    }
}

impl Interaction for ScopeBuildingPass {
    type Error = CannotLinkError;

    fn install(ctx: &mut Ctx) {
        ctx.register(Self::link_scopes.in_set(ScopeBuildingPass));
        ctx.register(Self::wrap_system(Self::system).in_set(ScopeBuildingPass));
    }
}

impl ScopeBuildingPass {
    fn spawn_scope(entity: ScopeUnion, commands: &mut Commands) -> Entity {
        let name = match &*entity {
            ScopeUnionItem::FileDecl(_) => None,
            ScopeUnionItem::ModDecl(x) => Some(x.name().clone()),
            ScopeUnionItem::StructDecl(x) => Some(x.name().clone()),
            ScopeUnionItem::EnumDecl(x) => Some(x.name().clone()),
            ScopeUnionItem::FuncDecl(x) => Some(x.name().clone()),
            ScopeUnionItem::Lambda(_) => None,
            ScopeUnionItem::Exprs(_) => None,
            ScopeUnionItem::IfExpr(_) => None,
        };

        let mut scope_entity = if let Some(name) = name {
            commands.spawn((Scope::new(entity.id), name))
        } else {
            commands.spawn(Scope::new(entity.id))
        };

        scope_entity
            .add_one_related::<Scoped>(entity.id.entity())
            .id()
    }

    /// If the current node is a start of scope, spawn new scope.
    ///
    /// Otherwise, propagate `scoped` component from parent if it has one
    fn system(
        unscoped: Populated<(AnyNodeRef, Option<&ChildOf>), (Without<Scoped>, With<Node>)>,
        scoped: Query<&Scoped>,
        mut phase: ResMut<CurrentPhase>,
        mut commands: Commands,
    ) -> crate::Result<CannotLinkError> {
        **phase = Phase::ScopeBuilding;
        let mut processed_any = false;
        let mut last_unprocessed = None;

        for (node, parent) in unscoped.iter() {
            if let Some(node_enum) = node.into_enum() {
                Self::spawn_scope(node_enum, &mut commands);
                processed_any = true;
            } else if let Some(parent_scope) = parent.and_then(|it| scoped.get(it.get()).ok()) {
                commands
                    .entity(parent_scope.0)
                    .add_one_related::<Scoped>(node.id().entity());
                processed_any = true;
            } else {
                last_unprocessed = Some(node.span());
            }
        }

        if let Some(last_unprocessed) = last_unprocessed {
            if !processed_any {
                fail(CannotLinkError(last_unprocessed))?;
            }
        }

        done()
    }

    fn link_scopes(query: Query<(Option<&ChildOf>, &Scoped)>, mut commands: Commands) {
        for (parent, scoped) in query.iter() {
            let Some(parent) = parent else { continue };
            let Ok((_, parent_scoped)) = query.get(parent.get()) else {
                continue;
            };

            if scoped.0 != parent_scoped.0 {
                commands.entity(parent_scoped.0).add_child(scoped.0);
            }
        }
    }
}
