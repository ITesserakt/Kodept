use crate::utils::{LogSystemEx, ReportSystemEx};
use bevy_ecs::message::MessageRegistry;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QuerySingleError;
use bevy_ecs::relationship::Relationship;
use bevy_ecs::schedule::ScheduleLabel;
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Node, RequireProperty, SourceSpan};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::consts::Const;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::{FileDecl, ModDecl};
use kodept_ast_nodes::function::FuncDecl;
use kodept_ast_nodes::top_level::{EnumConst, EnumDecl, StructDecl};
use kodept_ast_nodes::types::{NonTyParam, TyParam};
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, MessageBehaviour, Severity};
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Copy, Clone, ScheduleLabel, PartialEq, Eq, Hash)]
pub(super) struct SymbolResolution;

impl SymbolResolution {
    pub(super) fn configure(world: &mut World, schedule: &mut Schedule) {
        #[cfg(feature = "parallel")]
        schedule.set_executor_kind(bevy_ecs::schedule::ExecutorKind::MultiThreaded);

        MessageRegistry::register_message::<SpawnSymbolMessage>(world);

        schedule
            .add_systems(
                (
                    (
                        spawn_named_scope::<FileDecl>("root".into()).trace_completion(),
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
            .add_systems(emit_event::<ScopesBuiltEvent>.run_if(condition_changed_to(
                false,
                any_match_filter::<(With<Node>, Without<InScope>)>,
            )));

        schedule
            .add_systems((
                spawn_symbol(|_: &VarDecl| SymbolKind::Variable).trace_completion(),
                spawn_symbol(|_: &EnumConst| SymbolKind::Const).trace_completion(),
                spawn_symbol(|_: &TyParam| SymbolKind::Parameter).trace_completion(),
                spawn_symbol(|_: &NonTyParam| SymbolKind::Parameter).trace_completion(),
                spawn_symbol(|c: &Const| match c {
                    Const::Fn => SymbolKind::Function,
                    Const::Struct => SymbolKind::Type,
                    Const::Enum => SymbolKind::Type,
                    Const::Value => SymbolKind::Const,
                })
                .trace_completion(),
            ))
            .add_systems(
                populate_symbol_table
                    .trace_completion()
                    .extract_reports()
                    .run_if(on_message::<SpawnSymbolMessage>),
            );

        world.add_observer(link_scopes.trace_completion());
        world.add_observer(ensure_one_root_scope.trace_completion().extract_reports());
    }
}

#[derive(Debug, Event, Default)]
struct ScopesBuiltEvent;
#[derive(Debug, Event, Default)]
struct ScopesLinkedEvent;
#[derive(Debug, Message)]
struct SpawnSymbolMessage {
    entity: Entity,
    kind: SymbolKind,
}
#[derive(Debug)]
struct CannotLinkError(SourceSpan, &'static str);
#[derive(Debug)]
struct MultipleRootScopes(Vec<(SourceSpan, &'static str)>);
#[derive(Debug)]
enum SymbolErrors {
    NameNotFound(SourceSpan),
    SymbolTableNotFound(Entity),
    Duplicated {
        scope_name: Option<Name>,
        symbol_name: Name,
        scope_span: SourceSpan,
        current_symbol_span: SourceSpan,
        previous_symbol_span: SourceSpan,
    },
}

#[derive(Debug, Component)]
#[component(immutable)]
#[require(SymbolTable, Scoping)]
struct Scope {
    starts_from: Entity,
}

#[derive(Debug, Component, Default)]
struct SymbolTable {
    symbols: HashMap<(Name, SymbolKind), Symbol>,
    order: Vec<Entity>,
}

#[derive(Debug)]
struct Symbol {
    kind: SymbolKind,
    bound_to: Entity,
}

#[derive(Debug, Component, PartialEq)]
#[relationship(relationship_target = Scoping)]
#[component(immutable)]
/// Attaches to the ast nodes and points to appropriate enclosing scope
struct InScope(Entity);

#[derive(Debug, Component, Default)]
#[relationship_target(relationship = InScope)]
/// Attaches to the scope entity and describes a set of ast nodes that belongs to this scope
struct Scoping(Vec<Entity>);

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// Separates all symbols in one table into different kinds
enum SymbolKind {
    Type,
    Variable,
    Parameter,
    Function,
    Const,
}

fn emit_event<T: for<'a> Event<Trigger<'a>: Default> + Default>(mut commands: Commands) {
    commands.trigger(T::default())
}

fn spawn_scope<T: ASTNode>(
    query: Populated<(Entity, Option<&Name>), (Without<InScope>, With<T>)>,
    mut commands: Commands,
) {
    for (id, name) in query {
        if let Some(name) = name {
            commands
                .spawn((Scope { starts_from: id }, name.clone()))
                .add_one_related::<InScope>(id);
        } else {
            commands
                .spawn(Scope { starts_from: id })
                .add_one_related::<InScope>(id);
        }
    }
}

fn spawn_named_scope<T: ASTNode>(
    name: Name,
) -> impl FnMut(Populated<Entity, (Without<InScope>, With<T>)>, Commands) {
    move |query, mut commands| {
        for id in query {
            commands
                .spawn((Scope { starts_from: id }, name.clone()))
                .add_one_related::<InScope>(id);
        }
    }
}

fn spawn_symbol<T: ASTNode + RequireProperty<Name>>(
    kind: fn(&T) -> SymbolKind,
) -> impl FnMut(Populated<(Entity, &T), (With<Node>, Added<InScope>)>, MessageWriter<SpawnSymbolMessage>)
{
    move |query, mut writer| {
        writer.write_batch(query.into_iter().map(|it| SpawnSymbolMessage {
            entity: it.0,
            kind: kind(it.1),
        }));
    }
}

fn populate_symbol_table(
    mut messages: MessageReader<SpawnSymbolMessage>,
    nodes: Query<(&Name, &InScope), With<Node>>,
    spans: Query<&SourceSpan, With<Node>>,
    mut tables: Query<(&mut SymbolTable, Option<&Name>, &Scope)>,
) -> Result<(), Vec<SymbolErrors>> {
    let mut errors = vec![];
    for spawned_symbol in messages.read() {
        let span = *spans.get(spawned_symbol.entity).unwrap();
        let Ok((name, scope_id)) = nodes.get(spawned_symbol.entity) else {
            errors.push(SymbolErrors::NameNotFound(span));
            continue;
        };
        let Ok((mut table, scope_name, scope)) = tables.get_mut(scope_id.0) else {
            errors.push(SymbolErrors::SymbolTableNotFound(scope_id.0));
            continue;
        };

        match table.symbols.entry((name.clone(), spawned_symbol.kind)) {
            Entry::Occupied(mut entry) if entry.get().bound_to == spawned_symbol.entity => {
                entry.get_mut().kind = spawned_symbol.kind;
            }
            Entry::Occupied(entry) => {
                errors.push(SymbolErrors::Duplicated {
                    scope_name: scope_name.cloned(),
                    symbol_name: name.clone(),
                    scope_span: *spans.get(scope.starts_from).unwrap(),
                    current_symbol_span: *spans.get(spawned_symbol.entity).unwrap(),
                    previous_symbol_span: *spans.get(entry.get().bound_to).unwrap(),
                });
            }
            Entry::Vacant(entry) => {
                entry.insert(Symbol {
                    kind: spawned_symbol.kind,
                    bound_to: spawned_symbol.entity,
                });
            }
        };
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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

fn ensure_one_root_scope(
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

impl IntoSpannedReportMessage for SymbolErrors {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        match self {
            SymbolErrors::NameNotFound(x) => Diagnostic::new(Severity::Bug)
                .with_message("Cannot create a symbol for unnamed node")
                .with_primary_label("unnamed node", x),
            SymbolErrors::SymbolTableNotFound(x) => Diagnostic::new(Severity::Bug)
                .with_message("Cannot find corresponding symbol table at scope")
                .with_note(format!("scope id: {x}")),
            SymbolErrors::Duplicated {
                scope_name,
                symbol_name,
                scope_span,
                current_symbol_span,
                previous_symbol_span,
            } => {
                let scope_name_message = match scope_name {
                    Some(name) => Cow::Owned(format!("in scope `{name}`")),
                    None => "in scope".into(),
                };
                Diagnostic::new(Severity::Error)
                    .with_message(format!(
                        "Element with name `{}` already defined",
                        symbol_name
                    ))
                    .with_primary_label("", current_symbol_span)
                    .with_secondary_label("previous declaration", previous_symbol_span)
                    .with_secondary_label(scope_name_message, scope_span)
            }
        }
    }
}
