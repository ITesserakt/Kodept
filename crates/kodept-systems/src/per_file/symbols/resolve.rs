use crate::per_file::ast_normalization::InModule;
use crate::per_file::symbols::{SymbolKind, SymbolTable};
use crate::source::collection::Reporter;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::lifecycle::Add;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::{Children, Entity, On, With};
use bevy_ecs::query::{Has, Or, QueryData, Without};
use bevy_ecs::system::{Commands, Query};
use kodept_ast::Str;
use kodept_ast::export::Component;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast::properties::SourceSpan;
use kodept_ast_nodes::{
    Module, Path, ResolvedTypeAnnotation, TypeAnnotation, UserFunction, UserType, Value,
};
use kodept_core::code_point::Span;
use kodept_report::message::{Diagnostic, Severity};
use kodept_report::traits::IntoSpannedReportMessage;
use kodept_report_macros::Report;
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::iter::Peekable;
use std::marker::PhantomData;

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct ResolvedTo(NodeId, SymbolKind);

#[derive(Debug)]
enum Decision<A, R> {
    Next,
    Reject(R),
    Accept(A),
}

#[derive(Debug)]
enum SearchResult<A, R> {
    Rejected(R),
    Accepted(NodeId, A),
}

#[derive(Debug, Component)]
#[component(immutable, storage = "SparseSet")]
/// Marks nodes that inhibits resolution until node with [`Opaque`] component is found
pub(super) struct Passthrough;
#[derive(Debug, Component)]
#[component(immutable, storage = "SparseSet")]
/// Marks nodes that enabled resolution back
pub(super) struct Opaque;

#[derive(Debug)]
struct UnresolvedReference<I> {
    name: Str,
    span: Span,
    note: I,
}

#[derive(Debug, Report)]
#[severity("bug")]
#[message("Reference `{}` is not resolved still", self.id)]
struct UnexpectedUnresolvedReference {
    id: NodeId<Value>,
    #[primary_label("expected this to be resolved")]
    span: Span,
}

#[derive(Debug)]
enum Layered<T> {
    Layer,
    Value(T),
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Reject<'a> {
    SegmentMismatch {
        expected: &'a str,
    },
    NotFound {
        scope_name: Name,
        symbol_name: &'a str,
    },
    Exhausted,
    UnnamedScope,
    Passthrough,
}

#[derive(Debug, Default)]
struct Rejects<'a>(HashSet<Reject<'a>>);

impl<I> IntoSpannedReportMessage for UnresolvedReference<I>
where
    I: IntoIterator<Item: Display>,
{
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        let diagnostic = Diagnostic::new(Severity::Error)
            .with_message(format!("Cannot resolve reference `{}`", self.name))
            .with_primary_label("not found in scope", self.span);

        self.note
            .into_iter()
            .fold(diagnostic, |acc, next| acc.with_note(next.to_string()))
    }
}

impl Display for Reject<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Reject::SegmentMismatch { expected } => write!(f, "Unknown segment `{expected}`"),
            Reject::NotFound {
                scope_name,
                symbol_name,
            } => write!(f, "`{scope_name}` doesn't have `{symbol_name}` defined"),
            Reject::Exhausted => write!(f, "Exhausted"),
            Reject::UnnamedScope => write!(f, "UnnamedScope"),
            Reject::Passthrough => write!(f, "Cannot access definitions of outer scope"),
        }
    }
}

impl PartialOrd for Reject<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Reject<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Reject::NotFound { .. }, Reject::NotFound { .. }) => Ordering::Equal,
            (Reject::SegmentMismatch { .. }, Reject::SegmentMismatch { .. }) => Ordering::Equal,
            (Reject::Passthrough, Reject::Passthrough) => Ordering::Equal,

            (Reject::Passthrough, Reject::NotFound { .. }) => Ordering::Equal,
            (Reject::NotFound { .. }, Reject::Passthrough) => Ordering::Equal,

            (Reject::NotFound { .. }, _) => Ordering::Greater,
            (_, Reject::NotFound { .. }) => Ordering::Less,

            (Reject::SegmentMismatch { .. }, _) => Ordering::Greater,
            (_, Reject::SegmentMismatch { .. }) => Ordering::Less,

            _ => Ordering::Equal,
        }
    }
}

impl<'a> Extend<Reject<'a>> for Rejects<'a> {
    fn extend<T: IntoIterator<Item = Reject<'a>>>(&mut self, iter: T) {
        for reject in iter {
            match self.0.iter().next().map(|it| reject.cmp(it)) {
                None => _ = self.0.insert(reject),
                Some(Ordering::Equal) => _ = self.0.insert(reject),
                Some(Ordering::Less) => {}
                Some(Ordering::Greater) => {
                    self.0.clear();
                    self.0.insert(reject);
                }
            };
        }
    }
}

impl<'a> IntoIterator for Rejects<'a> {
    type Item = Reject<'a>;
    type IntoIter = std::collections::hash_set::IntoIter<Reject<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn walk_up<'s, T, A, R, P>(
    ancestors: Query<&ChildOf>,
    mut properties: Query<'_, 's, T>,
    start: impl Erase<Entity>,
    mut control: impl FnMut(T::Item<'_, 's>) -> Decision<A, R>,
) -> SearchResult<A, P>
where
    T: QueryData,
    R: Ord,
    P: Default + Extend<R>,
{
    let mut current = start.erase();
    let mut rejects = P::default();
    loop {
        let Ok(properties) = properties.get_mut(current) else {
            return SearchResult::Rejected(rejects);
        };
        match control(properties) {
            Decision::Next => {
                let Ok(ChildOf(parent)) = ancestors.get(current) else {
                    return SearchResult::Rejected(rejects);
                };
                current = *parent;
            }
            Decision::Reject(reject) => {
                let Ok(ChildOf(parent)) = ancestors.get(current) else {
                    return SearchResult::Rejected(rejects);
                };
                current = *parent;
                rejects.extend([reject]);
            }
            Decision::Accept(accept) => return SearchResult::Accepted(current.into(), accept),
        }
    }
}

trait WalkDownController<T> {
    type Accept;
    type Reject;

    fn on_iteration(&mut self, item: T) -> Decision<Self::Accept, Self::Reject>;
    fn on_layer(&mut self);
}

fn walk_down<'s, T, A, R, P>(
    descendants: Query<Option<&Children>>,
    mut properties: Query<'_, 's, T>,
    start: impl Erase<Entity>,
    mut control: impl for<'w> WalkDownController<T::Item<'w, 's>, Accept = A, Reject = R>,
) -> SearchResult<A, P>
where
    T: QueryData,
    R: Ord,
    P: Default + Extend<R>,
{
    let mut stack = VecDeque::from([Layered::Value(start.erase()), Layered::Layer]);
    let mut rejects = P::default();
    loop {
        match stack.pop_front() {
            None => return SearchResult::Rejected(rejects),
            Some(Layered::Layer) if stack.is_empty() => control.on_layer(),
            Some(Layered::Layer) => {
                control.on_layer();
                stack.push_back(Layered::Layer);
            }
            Some(Layered::Value(id)) => {
                let Ok(properties) = properties.get_mut(id) else {
                    return SearchResult::Rejected(rejects);
                };

                match control.on_iteration(properties) {
                    Decision::Next => stack.extend(
                        descendants
                            .get(id)
                            .iter()
                            .flat_map(|it| it.iter())
                            .flat_map(|it| it.iter())
                            .map(|it| Layered::Value(*it)),
                    ),
                    Decision::Reject(reject) => rejects.extend([reject]),
                    Decision::Accept(accept) => return SearchResult::Accepted(id.into(), accept),
                }
            }
        }
    }
}

fn add_on_add<T: Component, U: Component>(
    what: impl Fn() -> T,
    _: PhantomData<U>,
) -> impl Fn(On<Add, U>, Commands) {
    move |on_add, mut commands| {
        commands.entity(on_add.entity).insert(what());
    }
}

pub(super) fn add_passthrough_markers(
    query: Query<
        NodeId,
        Or<(
            With<UserFunction<TypeAnnotation>>,
            With<UserFunction<ResolvedTypeAnnotation>>,
        )>,
    >,
    mut commands: Commands,
) {
    for id in query {
        commands.entity(id.entity()).insert(Passthrough);
    }
    commands.add_observer(add_on_add(
        || Passthrough,
        PhantomData::<UserFunction<TypeAnnotation>>,
    ));
    commands.add_observer(add_on_add(
        || Passthrough,
        PhantomData::<UserFunction<ResolvedTypeAnnotation>>,
    ));
}

pub(super) fn add_opaque_markers(
    query: Query<NodeId, Or<(With<UserType>, With<Module>)>>,
    mut commands: Commands,
) {
    for id in query {
        commands.entity(id.entity()).insert(Opaque);
    }
    commands.add_observer(add_on_add(|| Opaque, PhantomData::<UserType>));
    commands.add_observer(add_on_add(|| Opaque, PhantomData::<Module>));
}

fn resolve_local_value<'i>(
    start_id: impl Erase,
    ident: &'i Str,
    ancestors: Query<&ChildOf>,
    properties: Query<(Option<&SymbolTable>, Has<Passthrough>, Has<Opaque>)>,
) -> SearchResult<(NodeId, SymbolKind), Rejects<'i>> {
    let mut boundary = false;

    walk_up(
        ancestors,
        properties,
        start_id,
        |(table, should_pass_through, should_block)| {
            match (should_pass_through, should_block, boundary) {
                (true, false, false) => boundary = true,
                (_, false, true) => return Decision::Reject(Reject::Passthrough),
                (false, true, _) => boundary = false,
                _ => {}
            };

            table
                .and_then(|it| it.lookup(ident))
                .map_or(Decision::Next, |item| Decision::Accept(item))
        },
    )
}

fn walk_down_controller<'i, I: ExactSizeIterator<Item = &'i Str>>(
    segments: I,
    ident: &'i Str,
) -> impl for<'w> WalkDownController<
    (Option<&'w Name>, Option<&'w SymbolTable>),
    Accept = (NodeId, SymbolKind),
    Reject = Reject<'i>,
> {
    struct Capture<'i, I: Iterator<Item = &'i Str>> {
        segments: Peekable<I>,
        current: Option<&'i Str>,
        is_last: bool,
        ident: &'i Str,
    }

    impl<'i, I> WalkDownController<(Option<&Name>, Option<&SymbolTable>)> for Capture<'i, I>
    where
        I: Iterator<Item = &'i Str>,
    {
        type Accept = (NodeId, SymbolKind);
        type Reject = Reject<'i>;

        fn on_iteration(
            &mut self,
            (name, table): (Option<&Name>, Option<&SymbolTable>),
        ) -> Decision<Self::Accept, Self::Reject> {
            let &Some(expected) = &self.current else {
                return Decision::Reject(Reject::Exhausted);
            };
            let Some(name) = name else {
                return Decision::Reject(Reject::UnnamedScope);
            };
            if expected.as_ref() != name.as_ref() {
                return Decision::Reject(Reject::SegmentMismatch {
                    expected: expected.as_ref(),
                });
            }
            match self.is_last {
                false => Decision::Next,
                true => match table.and_then(|it| it.lookup(self.ident)) {
                    Some(item) => Decision::Accept(item),
                    None => Decision::Reject(Reject::NotFound {
                        scope_name: name.clone(),
                        symbol_name: self.ident.as_ref(),
                    }),
                },
            }
        }

        fn on_layer(&mut self) {
            self.current = self.segments.next();
            self.is_last = self.segments.peek().is_none();
        }
    }

    let mut segments = segments.peekable();
    let current = segments.next();
    let is_last = segments.peek().is_none();

    Capture {
        segments: segments.peekable(),
        current,
        is_last,
        ident,
    }
}

fn resolve_global_value_with_context<'i>(
    module_id: NodeId<Module>,
    context: &'i [Str],
    ident: &'i Str,
    descendants: Query<Option<&Children>>,
    properties: Query<(Option<&Name>, Option<&SymbolTable>)>,
) -> SearchResult<(NodeId, SymbolKind), Rejects<'i>> {
    walk_down(
        descendants,
        properties,
        module_id,
        walk_down_controller(context.iter().peekable(), ident),
    )
}

fn resolve_local_value_with_context<'i>(
    start_id: impl Erase,
    context: &'i [Str],
    ident: &'i Str,
    ancestors: Query<&ChildOf>,
    descendants: Query<Option<&Children>>,
    names: Query<(Option<&Name>, Has<Passthrough>, Has<Opaque>)>,
    properties: Query<(Option<&Name>, Option<&SymbolTable>)>,
) -> SearchResult<(NodeId, SymbolKind), Rejects<'i>> {
    let first = context.first().unwrap();
    let mut boundary = false;

    let result = walk_up(
        ancestors,
        names,
        start_id,
        |(name, should_pass_through, should_block)| {
            match (should_pass_through, should_block, boundary) {
                (true, false, false) => boundary = true,
                (_, false, true) => return Decision::Reject(Reject::Passthrough),
                (false, true, _) => boundary = false,
                _ => {}
            };

            match name {
                None => return Decision::Next,
                Some(name) if name.as_ref() == first.as_ref() => Decision::Accept(()),
                Some(_) => Decision::Next,
            }
        },
    );
    let start = match result {
        SearchResult::Rejected(passthrough) => return SearchResult::Rejected(passthrough),
        SearchResult::Accepted(id, _) => id,
    };

    walk_down(
        descendants,
        properties,
        start,
        walk_down_controller(context.iter().peekable(), ident),
    )
}

pub(super) fn resolve_values(
    values: Query<(NodeId<Value>, &Value, &InModule, &SourceSpan)>,
    ancestors: Query<&ChildOf>,
    descendants: Query<Option<&Children>>,
    table_and_passthrough: Query<(Option<&SymbolTable>, Has<Passthrough>, Has<Opaque>)>,
    name_and_table: Query<(Option<&Name>, Option<&SymbolTable>)>,
    name_and_passthrough: Query<(Option<&Name>, Has<Passthrough>, Has<Opaque>)>,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, value, &InModule(module_id), span) in values {
        let Value { path, ident } = value;

        let result = match &path {
            Path {
                is_global: false,
                segments,
            } if segments.is_empty() => {
                resolve_local_value(id, ident, ancestors, table_and_passthrough)
            }
            Path {
                is_global: true,
                segments,
            } => resolve_global_value_with_context(
                module_id,
                segments,
                ident,
                descendants,
                name_and_table,
            ),
            Path {
                is_global: false,
                segments,
            } => resolve_local_value_with_context(
                id,
                segments,
                ident,
                ancestors,
                descendants,
                name_and_passthrough,
                name_and_table,
            ),
        };

        match result {
            SearchResult::Rejected(rejects) => reporter.report(UnresolvedReference {
                name: ident.clone(),
                span: span.0,
                note: rejects,
            }),
            SearchResult::Accepted(_, item) => {
                commands
                    .entity(id.entity())
                    .insert(ResolvedTo(item.0, item.1));
            }
        }
    }
}

pub(super) fn ensure_all_values_resolved(
    query: Query<(NodeId<Value>, &SourceSpan), Without<ResolvedTo>>,
    mut reporter: Reporter,
) {
    for (id, span) in query {
        reporter.report(UnexpectedUnresolvedReference { id, span: span.0 });
    }
}
