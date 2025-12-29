mod resolution;
mod scopes;
mod tables;

use crate::utils::{LogSystemEx, ReportSystemEx};
use bevy_ecs::message::MessageRegistry;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use hashbrown::Equivalent;
use hashbrown::hash_map::{Entry, HashMap};
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Node, SourceSpan};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::code_flow::IfExpr;
use kodept_ast_nodes::consts::Const;
use kodept_ast_nodes::expression::{Exprs, Lambda};
use kodept_ast_nodes::file::{FileDecl, ModDecl};
use kodept_ast_nodes::function::FuncDecl;
use kodept_ast_nodes::term::Ref;
use kodept_ast_nodes::top_level::{EnumConst, EnumDecl, StructDecl};
use kodept_ast_nodes::types::{NonTyParam, Ty, TyParam};
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, MessageBehaviour, Severity};
use resolution::*;
use scopes::*;
use std::borrow::Cow;
use kodept_report_macros::Report;
use tables::*;

#[derive(Debug, Copy, Clone, ScheduleLabel, PartialEq, Eq, Hash)]
pub(super) struct SymbolResolution;

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Copy, Clone)]
struct SymbolSpawningSystems;

impl SymbolResolution {
    pub(super) fn configure(world: &mut World, schedule: &mut Schedule) {
        #[cfg(feature = "parallel")]
        schedule.set_executor_kind(bevy_ecs::schedule::ExecutorKind::MultiThreaded);

        MessageRegistry::register_message::<SpawnSymbolMessage>(world);
        MessageRegistry::register_message::<ResolveRefAtMessage>(world);

        schedule
            .add_systems(
                (
                    (
                        spawn_scope::<FileDecl>(ScopeSpawnParams::default().with_name("root"))
                            .trace_completion(),
                        spawn_scope::<Exprs>(ScopeSpawnParams::default()).trace_completion(),
                        spawn_scope::<ModDecl>(ScopeSpawnParams::default()).trace_completion(),
                        spawn_scope::<StructDecl>(ScopeSpawnParams::default()).trace_completion(),
                        spawn_scope::<EnumDecl>(ScopeSpawnParams::default()).trace_completion(),
                        spawn_scope::<FuncDecl>(ScopeSpawnParams::default()).trace_completion(),
                        spawn_scope::<Lambda>(ScopeSpawnParams::default()).trace_completion(),
                        spawn_scope::<IfExpr>(ScopeSpawnParams::default()).trace_completion(),
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
            .add_systems(
                (
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
                )
                    .in_set(SymbolSpawningSystems),
            )
            .add_systems(
                populate_symbol_table
                    .trace_completion()
                    .extract_reports()
                    .run_if(on_message::<SpawnSymbolMessage>),
            );

        schedule
            .add_systems(
                (
                    start_resolution::<Ref>.trace_completion(),
                    start_resolution::<Ty>.trace_completion(),
                )
                    .run_if(condition_changed_to(
                        false,
                        on_message::<SpawnSymbolMessage>,
                    )),
            )
            .add_systems(mark_refs_as_deferred.trace_completion().run_if(run_once))
            .add_systems(
                process_resolve_messages
                    .trace_completion()
                    .extract_reports()
                    .run_if(on_message::<ResolveRefAtMessage>),
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
struct ResolveRefAtMessage {
    ref_id: Entity,
    scope_id: Entity,
}
#[derive(Debug, Message)]
struct SpawnSymbolMessage {
    entity: Entity,
    kind: SymbolKind,
}

#[derive(Debug, Report)]
#[severity("bug")]
#[message("Cannot create new scope or link with any other")]
#[note("Possible out-of-tree nodes?")]
#[fail_fast("Scoping failed")]
struct CannotLinkError {
    #[primary_label("unprocessed node: {}", self.node_kind)]
    node_location: SourceSpan,
    node_kind: String,
}

#[derive(Debug)]
struct MultipleRootScopes(Vec<(SourceSpan, String)>);

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

#[derive(Debug, Report)]
#[severity("error")]
#[message("Cannot resolve reference `{}`", self.reference_name)]
struct UnresolvedReference {
    reference_name: Name,
    #[primary_label("not found in scope")]
    reference_span: SourceSpan,
}

#[derive(Debug, Component)]
#[component(immutable)]
#[require(SymbolTable, Scoping)]
struct Scope {
    starts_from: Entity,
}

#[derive(Debug)]
struct ScopeSpawnParams {
    override_name: Option<Name>,
}

impl ScopeSpawnParams {
    const fn default() -> Self {
        Self {
            override_name: None,
        }
    }

    fn with_name(self, name: impl Into<Name>) -> Self {
        Self {
            override_name: Some(name.into()),
            ..self
        }
    }
}

#[derive(Debug, Component, Default)]
struct SymbolTable {
    symbols: HashMap<SymbolDescriptor, Symbol>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct SymbolDescriptor {
    name: Name,
    kind: SymbolKind,
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct SymbolDescriptorView<'a> {
    name: &'a str,
    kind: &'a SymbolKind,
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

#[derive(Debug, Component)]
/// Defers reference resolution
struct DeferRefResolution;

#[derive(Debug, Component)]
#[component(immutable)]
struct SymbolUsage {
    kind: SymbolKind,
    symbol_scope_id: Entity,
}

fn emit_event<T: for<'a> Event<Trigger<'a>: Default> + Default>(mut commands: Commands) {
    commands.trigger(T::default())
}

impl Equivalent<SymbolDescriptor> for SymbolDescriptorView<'_> {
    fn equivalent(&self, key: &SymbolDescriptor) -> bool {
        self.kind == &key.kind && self.name == key.name.as_str()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn hash_value(value: &impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_hash_equivalence() {
        let a = SymbolDescriptor {
            name: Name::from("value a"),
            kind: SymbolKind::Type,
        };

        let b = SymbolDescriptorView {
            name: "value a",
            kind: &SymbolKind::Type,
        };

        assert!(Equivalent::equivalent(&b, &a));
        assert_eq!(hash_value(&a), hash_value(&b));
    }
}
