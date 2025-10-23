use crate::utils::{LogSystemEx, ReportSystemEx};
use bevy_ecs::prelude::*;
use bevy_ecs::query::QuerySingleError;
use bevy_ecs::relationship::Relationship;
use bevy_ecs::schedule::ScheduleLabel;
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Node, SourceSpan};
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::{FileDecl, ModDecl};
use kodept_ast_nodes::function::FuncDecl;
use kodept_ast_nodes::top_level::{EnumDecl, StructDecl};
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, MessageBehaviour, Severity};

#[derive(Debug, Copy, Clone, ScheduleLabel, PartialEq, Eq, Hash)]
pub(super) struct SymbolResolution;

impl SymbolResolution {
    pub(super) fn configure(world: &mut World, schedule: &mut Schedule) {
        #[cfg(feature = "parallel")]
        schedule.set_executor_kind(bevy_ecs::schedule::ExecutorKind::MultiThreaded);

        schedule
            .add_systems(
                (
                    (
                        spawn_scope::<FileDecl>.trace_completion(),
                        spawn_scope::<ModDecl>.trace_completion(),
                        spawn_scope::<StructDecl>.trace_completion(),
                        spawn_scope::<EnumDecl>.trace_completion(),
                        spawn_scope::<FuncDecl>.trace_completion(),
                        spawn_scope::<Lambda>.trace_completion(),
                        spawn_scope::<Exprs>.trace_completion(),
                        spawn_scope::<IfExpr>.trace_completion(),
                    ),
                    propagate_scopes.trace_completion().extract_reports(),
                )
                    .chain(),
            )
            .add_systems(emit_event::<ScopesBuilt>.run_if(condition_changed_to(
                false,
                any_match_filter::<(With<Node>, Without<InScope>)>,
            )));

        world.add_observer(link_scopes.trace_completion());
        world.add_observer(ensure_one_root_scope.trace_completion().extract_reports());
    }
}

#[derive(Debug, Event, Default)]
struct ScopesBuilt;
#[derive(Debug, Event, Default)]
struct ScopesLinked;
#[derive(Debug)]
struct CannotLinkError(SourceSpan, &'static str);
#[derive(Debug)]
struct MultipleRootScopes(Vec<(SourceSpan, &'static str)>);

#[derive(Debug, Component)]
struct Scope {
    root_node: Entity,
}

#[derive(Debug, Component, PartialEq)]
#[relationship(relationship_target = Scoping)]
/// Attaches to the ast nodes and points to appropriate enclosing scope
struct InScope(Entity);

#[derive(Debug, Component, Default)]
#[relationship_target(relationship = InScope)]
/// Attaches to the scope entity and describes a set of ast nodes that belongs to this scope
struct Scoping(Vec<Entity>);

#[derive(Debug)]
/// Separates all symbols into different kinds
enum SymbolKind {}

fn emit_event<T: for<'a> Event<Trigger<'a>: Default> + Default>(mut commands: Commands) {
    commands.trigger(T::default())
}

fn spawn_scope<T: ASTNode>(
    query: Populated<(Entity, Option<&Name>), (Without<InScope>, With<T>)>,
    mut commands: Commands,
) {
    for (id, name) in query {
        let scope = if let Some(name) = name {
            commands
                .spawn((Scoping::default(), Scope { root_node: id }, name.clone()))
                .id()
        } else {
            commands
                .spawn((Scoping::default(), Scope { root_node: id }))
                .id()
        };
        commands.entity(scope).add_one_related::<InScope>(id);
    }
}

/// Sets scope of a node to be equal to the parent one if not exist
fn propagate_scopes(
    query: Populated<(Entity, Option<&ChildOf>, &SourceSpan, &Node), Without<InScope>>,
    scopes: Query<&InScope>,
    mut commands: Commands,
) -> Result<(), CannotLinkError> {
    let mut any_processed = false;
    let mut last_unprocessed = None;
    for (id, parent, span, node) in query {
        let Some(scope) = parent.and_then(|it| scopes.get(it.get()).ok()) else {
            last_unprocessed = Some((span, node.kind));
            continue;
        };
        // parent was scoped already
        any_processed = true;
        commands.entity(scope.0).add_one_related::<InScope>(id);
    }

    if let Some((span, kind)) = last_unprocessed
        && !any_processed
    {
        Err(CannotLinkError(*span, kind))
    } else {
        Ok(())
    }
}

fn link_scopes(
    _: On<ScopesBuilt>,
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
    commands.trigger(ScopesLinked);
}

fn ensure_one_root_scope(
    _: On<ScopesLinked>,
    query: Query<&Scope, Without<ChildOf>>,
    nodes: Query<(&SourceSpan, &Node)>,
) -> Result<(), MultipleRootScopes> {
    let Err(QuerySingleError::MultipleEntities(_)) = query.single() else {
        return Ok(());
    };

    Err(MultipleRootScopes(
        query
            .iter()
            .filter_map(|it| nodes.get(it.root_node).ok())
            .map(|it| (*it.0, it.1.kind))
            .collect(),
    ))
}

impl IntoSpannedReportMessage for CannotLinkError {
    type Message = Diagnostic;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Scoping failed")
    }

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Bug)
            .with_message("Cannot create new scope or link with any other")
            .with_note("Possible out-of-tree nodes?")
            .with_primary_label(format!("unprocessed node: {}", self.1), self.0)
    }
}

impl IntoSpannedReportMessage for MultipleRootScopes {
    type Message = Diagnostic;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Scoping failed")
    }

    fn into_message(self) -> Self::Message {
        let d = Diagnostic::new(Severity::Bug).with_message("Multiple root scopes are not allowed");
        self.0.into_iter().fold(d, |acc, next| {
            acc.with_primary_label(format!("root scope: {}", next.1), next.0)
        })
    }
}
