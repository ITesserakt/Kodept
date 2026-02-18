use crate::per_file::ast_normalization::InModule;
use crate::per_file::symbols::{SymbolKind, SymbolTable};
use crate::source::collection::Reporter;
use kodept_ast::Str;
use kodept_ast::prelude::{ASTNode, Erase, NodeId};
use kodept_ast::properties::{Lexeme, Name, SourceSpan};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::experimental::NodeModification;
use kodept_ast_nodes::{
    AnonFunction, CtorName, ForeignFunction, Module, Param, Path, PositionalParam, ResolvedType,
    ResolvedTypeAnnotation, TypeAnnotation, UnresolvedType, UserFunction, UserType, Value,
    ValueCtor, Variable,
};
use kodept_core::code_point::Span;
use kodept_core::structure::SpanBounds;
use kodept_ecs::component::Component;
use kodept_ecs::entity::Entity;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::hierarchy::{ChildOf, Children};
use kodept_ecs::lifecycle::Add;
use kodept_ecs::query::{Has, Or, QueryData, QueryFilter, With};
use kodept_ecs::system::{Commands, On, Query, Res, SystemParam};
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
struct UnresolvedReference<'a, I> {
    name: &'a str,
    span: Span,
    note: I,
}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Definition of `{}` expected to be a type, but it is a {}", self.name, self.actual_kind)]
struct SymbolIsNotType<'a> {
    #[secondary_label("is not a type")]
    span: Span,
    actual_kind: SymbolKind,
    name: &'a str,
    #[primary_label("required by a {} `{}`", self.ref_kind, self.ref_name)]
    ref_span: Span,
    ref_kind: &'static str,
    ref_name: &'a str,
}

#[derive(Debug, Report)]
#[severity("bug")]
#[message("Reference `{}` is not resolved still", self.id)]
struct UnexpectedUnresolvedReference {
    id: NodeId,
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

impl<I> IntoSpannedReportMessage for UnresolvedReference<'_, I>
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

#[derive(SystemParam, Copy, Clone)]
pub(super) struct Properties<'w, 's> {
    ancestors: Query<'w, 's, &'static ChildOf>,
    descendants: Query<'w, 's, Option<&'static Children>>,
    table_and_passthrough:
        Query<'w, 's, (Option<&'static SymbolTable>, Has<Passthrough>, Has<Opaque>)>,
    name_and_table: Query<'w, 's, (Option<&'static Name>, Option<&'static SymbolTable>)>,
    name_and_passthrough: Query<'w, 's, (Option<&'static Name>, Has<Passthrough>, Has<Opaque>)>,
}

fn resolve_ident_with_path<'i>(
    id: impl Erase,
    module_id: NodeId<Module>,
    path: &'i Path,
    ident: &'i Str,
    properties: Properties,
) -> SearchResult<(NodeId, SymbolKind), Rejects<'i>> {
    match &path {
        Path {
            is_global: false,
            segments,
        } if segments.is_empty() => resolve_local_value(
            id,
            ident,
            properties.ancestors,
            properties.table_and_passthrough,
        ),
        Path {
            is_global: true,
            segments,
        } => resolve_global_value_with_context(
            module_id,
            segments,
            ident,
            properties.descendants,
            properties.name_and_table,
        ),
        Path {
            is_global: false,
            segments,
        } => resolve_local_value_with_context(
            id,
            segments,
            ident,
            properties.ancestors,
            properties.descendants,
            properties.name_and_passthrough,
            properties.name_and_table,
        ),
    }
}

pub(super) fn resolve_values(
    values: Query<(NodeId<Value>, &Value, &InModule, &SourceSpan)>,
    properties: Properties,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, value, &InModule(module_id), span) in values {
        let Value { path, ident } = value;

        match resolve_ident_with_path(id, module_id, path, ident, properties) {
            SearchResult::Rejected(rejects) => reporter.report(UnresolvedReference {
                name: ident,
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

enum TypeResolutionError<'a> {
    Rejected(Rejects<'a>, &'a str),
    WrongKind(NodeId, SymbolKind, &'a str),
}

fn resolve_type_annotation<'i>(
    id: impl Erase + Copy,
    module_id: NodeId<Module>,
    annotation: &'i TypeAnnotation,
    properties: Properties,
) -> Result<ResolvedTypeAnnotation, TypeResolutionError<'i>> {
    match annotation {
        TypeAnnotation::Infer => Ok(ResolvedTypeAnnotation::Infer),
        TypeAnnotation::Named { path, ident } => {
            match resolve_ident_with_path(id, module_id, path, ident, properties) {
                SearchResult::Rejected(rejects) => {
                    Err(TypeResolutionError::Rejected(rejects, ident))
                }
                SearchResult::Accepted(_, (resolved_to, SymbolKind::Type)) => {
                    Ok(ResolvedTypeAnnotation::Named(resolved_to))
                }
                SearchResult::Accepted(_, (resolved_to, actual_kind)) => Err(
                    TypeResolutionError::WrongKind(resolved_to, actual_kind, ident),
                ),
            }
        }
        TypeAnnotation::Tuple(items) => {
            let items = items
                .iter()
                .map(|it| resolve_type_annotation(id, module_id, it, properties))
                .collect::<Result<_, _>>()?;
            Ok(ResolvedTypeAnnotation::Tuple(items))
        }
    }
}

fn resolve_type<'i>(
    id: impl Erase + Copy,
    module_id: NodeId<Module>,
    ty: &'i UnresolvedType,
    properties: Properties,
) -> Result<ResolvedType, TypeResolutionError<'i>> {
    match ty {
        UnresolvedType::Named { path, ident } => {
            match resolve_ident_with_path(id, module_id, path, ident, properties) {
                SearchResult::Rejected(rejects) => {
                    Err(TypeResolutionError::Rejected(rejects, ident))
                }
                SearchResult::Accepted(_, (resolved_to, SymbolKind::Type)) => {
                    Ok(ResolvedType::Named(resolved_to))
                }
                SearchResult::Accepted(_, (resolved_to, actual_kind)) => Err(
                    TypeResolutionError::WrongKind(resolved_to, actual_kind, ident),
                ),
            }
        }
        UnresolvedType::Tuple(items) => {
            let items = items
                .iter()
                .map(|it| resolve_type(id, module_id, it, properties))
                .collect::<Result<_, _>>()?;
            Ok(ResolvedType::Tuple(items))
        }
    }
}

pub(super) fn resolve_type_in_variables(
    variables: Query<(
        NodeId<Variable<TypeAnnotation>>,
        &Variable<TypeAnnotation>,
        &InModule,
        &Lexeme,
        &SourceSpan,
    )>,
    all_spans: Query<&SourceSpan>,
    syntax: Res<SyntaxResolver>,
    properties: Properties,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, variable, &InModule(module_id), lexeme, span) in variables {
        let return_type_span = syntax
            .try_get::<kodept_rlt::prelude::InitializedVariable>(lexeme.0)
            .ok()
            .and_then(|it| it.variable.assigned_type.as_ref())
            .map(|it| it.1.bounds())
            .unwrap_or(span.0);
        let modification = NodeModification::new(commands.reborrow(), id);
        match resolve_type_annotation(id, module_id, &variable.annotation, properties) {
            Ok(annotation) => {
                _ = modification.transmute(Variable {
                    mutable: variable.mutable,
                    name: variable.name.clone(),
                    annotation,
                })
            }
            Err(TypeResolutionError::Rejected(rejects, name)) => {
                reporter.report(UnresolvedReference {
                    span: return_type_span,
                    name,
                    note: rejects,
                })
            }
            Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                reporter.report(SymbolIsNotType {
                    name,
                    actual_kind,
                    span: all_spans
                        .get(resolved_to.entity())
                        .map(|it| it.0)
                        .unwrap_or_default(),
                    ref_span: return_type_span,
                    ref_kind: "explicit type annotation",
                    ref_name: variable.name.as_str(),
                })
            }
        }
    }
}

pub(super) fn resolve_types_in_user_functions(
    user_functions: Query<(
        NodeId<UserFunction<TypeAnnotation>>,
        &mut UserFunction<TypeAnnotation>,
        &Name,
        &InModule,
        &Lexeme,
        &SourceSpan,
    )>,
    all_spans: Query<&SourceSpan>,
    syntax: Res<SyntaxResolver>,
    properties: Properties,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, mut func, func_name, &InModule(module_id), lexeme, span) in user_functions {
        let modification = NodeModification::new(commands.reborrow(), id);
        let func_syntax = syntax
            .try_get::<kodept_rlt::prelude::BodiedFunction>(lexeme.0)
            .ok();
        let return_type_span = func_syntax
            .and_then(|it| it.return_type.as_ref())
            .map(|it| it.1.bounds())
            .unwrap_or(span.0);

        let return_type =
            match resolve_type_annotation(id, module_id, &func.return_type, properties) {
                Ok(annotation) => annotation,
                Err(TypeResolutionError::Rejected(rejects, name)) => {
                    reporter.report(UnresolvedReference {
                        name,
                        note: rejects,
                        span: return_type_span,
                    });
                    continue;
                }
                Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                    reporter.report(SymbolIsNotType {
                        name,
                        actual_kind,
                        span: all_spans
                            .get(resolved_to.entity())
                            .map(|it| it.0)
                            .unwrap_or_default(),
                        ref_span: return_type_span,
                        ref_kind: "return type",
                        ref_name: func_name.as_ref(),
                    });
                    continue;
                }
            };

        let params = func.params.drain(..).enumerate().map(|(index, param)| {
            let result = resolve_type_annotation(id, module_id, param.ty(), properties);
            let param_span = func_syntax
                .and_then(|it| it.params.as_ref())
                .and_then(|it| it.inner.get(index))
                .map(|it| it.bounds())
                .unwrap_or(span.0);

            let annotation = match result {
                Ok(x) => x,
                Err(TypeResolutionError::Rejected(rejects, name)) => {
                    reporter.report(UnresolvedReference {
                        name,
                        note: rejects,
                        span: param_span,
                    });
                    return None;
                }
                Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                    reporter.report(SymbolIsNotType {
                        name,
                        actual_kind,
                        span: all_spans
                            .get(resolved_to.entity())
                            .map(|it| it.0)
                            .unwrap_or_default(),
                        ref_span: param_span,
                        ref_kind: "parameter",
                        ref_name: param.name(),
                    });
                    return None;
                }
            };

            match param {
                Param::Positional { name, .. } => Some(Param::Positional {
                    name,
                    ty: annotation,
                }),
                Param::Named {
                    name,
                    default_expr_id,
                    ..
                } => Some(Param::Named {
                    name,
                    ty: annotation,
                    default_expr_id,
                }),
            }
        });
        let Some(params) = params.collect() else {
            continue;
        };

        modification.transmute(UserFunction {
            return_type,
            params,
        });
    }
}

pub(super) fn resolve_types_in_value_ctors(
    query: Query<(
        NodeId<ValueCtor<UnresolvedType>>,
        &mut ValueCtor<UnresolvedType>,
        &InModule,
        &SourceSpan,
    )>,
    all_spans: Query<&SourceSpan>,
    properties: Properties,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, mut ctor, &InModule(module_id), span) in query {
        let modification = NodeModification::new(commands.reborrow(), id);

        let params = ctor.params.drain(..).map(|param| {
            let result = resolve_type(id, module_id, param.ty(), properties);
            let ty = match result {
                Ok(x) => x,
                Err(TypeResolutionError::Rejected(rejects, name)) => {
                    reporter.report(UnresolvedReference {
                        name,
                        note: rejects,
                        span: span.0,
                    });
                    return None;
                }
                Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                    reporter.report(SymbolIsNotType {
                        name,
                        actual_kind,
                        ref_kind: "parameter",
                        ref_name: param.name(),
                        span: all_spans
                            .get(resolved_to.entity())
                            .map(|it| it.0)
                            .unwrap_or_default(),
                        ref_span: span.0,
                    });
                    return None;
                }
            };

            match param {
                Param::Positional { name, .. } => Some(Param::Positional { name, ty }),
                Param::Named {
                    name,
                    default_expr_id,
                    ..
                } => Some(Param::Named {
                    name,
                    default_expr_id,
                    ty,
                }),
            }
        });
        let Some(params) = params.collect() else {
            continue;
        };

        modification.transmute(ValueCtor {
            name: std::mem::replace(&mut ctor.name, CtorName::Inline),
            params,
        });
    }
}

pub(super) fn resolve_types_in_anon_functions(
    query: Query<(
        NodeId<AnonFunction<TypeAnnotation>>,
        &mut AnonFunction<TypeAnnotation>,
        &InModule,
        &Lexeme,
        &SourceSpan,
    )>,
    all_spans: Query<&SourceSpan>,
    properties: Properties,
    syntax: Res<SyntaxResolver>,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, mut func, &InModule(module_id), lexeme, span) in query {
        let func_syntax = syntax.try_get::<kodept_rlt::prelude::Lambda>(lexeme.0).ok();

        let return_type =
            match resolve_type_annotation(id, module_id, &func.return_type, properties) {
                Ok(x) => x,
                Err(TypeResolutionError::Rejected(rejects, name)) => {
                    reporter.report(UnresolvedReference {
                        name,
                        note: rejects,
                        span: span.0,
                    });
                    continue;
                }
                Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                    reporter.report(SymbolIsNotType {
                        span: all_spans
                            .get(resolved_to.entity())
                            .map(|it| it.0)
                            .unwrap_or_default(),
                        actual_kind,
                        name,

                        ref_span: Default::default(),
                        ref_kind: "parameter",
                        ref_name: "anonymous function",
                    });
                    continue;
                }
            };

        let params = func.params.drain(..).enumerate().map(|(index, it)| {
            let result = resolve_type_annotation(id, module_id, &it.ty, properties);
            let param_span = func_syntax
                .and_then(|it| it.binds.inner.get(index))
                .map(|it| it.bounds())
                .unwrap_or(span.0);

            let ty = match result {
                Ok(x) => x,
                Err(TypeResolutionError::Rejected(rejects, name)) => {
                    reporter.report(UnresolvedReference {
                        name,
                        note: rejects,
                        span: param_span,
                    });
                    return None;
                }
                Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                    reporter.report(SymbolIsNotType {
                        span: all_spans
                            .get(resolved_to.entity())
                            .map(|it| it.0)
                            .unwrap_or_default(),
                        name,
                        ref_span: param_span,
                        ref_kind: "parameter",
                        actual_kind,
                        ref_name: &it.name,
                    });
                    return None;
                }
            };

            Some(PositionalParam { name: it.name, ty })
        });
        let Some(params) = params.collect() else {
            continue;
        };

        let modification = NodeModification::new(commands.reborrow(), id);
        modification.transmute(AnonFunction {
            return_type,
            params,
        });
    }
}

pub(super) fn resolve_types_in_foreign_functions(
    query: Query<(
        NodeId<ForeignFunction<UnresolvedType>>,
        &mut ForeignFunction<UnresolvedType>,
        &Name,
        &InModule,
        &SourceSpan,
    )>,
    all_spans: Query<&SourceSpan>,
    properties: Properties,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, mut func, func_name, &InModule(module_id), span) in query {
        let return_type = match resolve_type(id, module_id, &func.return_type, properties) {
            Ok(x) => x,
            Err(TypeResolutionError::Rejected(rejects, name)) => {
                reporter.report(UnresolvedReference {
                    name,
                    note: rejects,
                    span: span.0,
                });
                continue;
            }
            Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                reporter.report(SymbolIsNotType {
                    name,
                    actual_kind,
                    span: all_spans
                        .get(resolved_to.entity())
                        .map(|it| it.0)
                        .unwrap_or_default(),
                    ref_kind: "return type",
                    ref_name: func_name.as_ref(),
                    ref_span: span.0,
                });
                continue;
            }
        };

        let params = func.params.drain(..).enumerate().map(|(index, it)| {
            match resolve_type(id, module_id, &it, properties) {
                Ok(x) => Some(x),
                Err(TypeResolutionError::Rejected(rejected, name)) => {
                    reporter.report(UnresolvedReference {
                        name,
                        note: rejected,
                        span: span.0,
                    });
                    None
                }
                Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                    reporter.report(SymbolIsNotType {
                        name,
                        actual_kind,
                        span: all_spans
                            .get(resolved_to.entity())
                            .map(|it| it.0)
                            .unwrap_or_default(),
                        ref_kind: "parameter",
                        ref_name: &index.to_string(),
                        ref_span: span.0,
                    });
                    None
                }
            }
        });
        let Some(params) = params.collect() else {
            continue;
        };

        let modification = NodeModification::new(commands.reborrow(), id);
        modification.transmute(ForeignFunction {
            return_type,
            params,
        });
    }
}

pub(super) fn ensure_absent<T: ASTNode, F: QueryFilter>(
    query: Query<(NodeId<T>, &SourceSpan), F>,
    mut reporter: Reporter,
) {
    for (id, span) in query {
        reporter.report(UnexpectedUnresolvedReference {
            id: id.cast(),
            span: span.0,
        });
    }
}
