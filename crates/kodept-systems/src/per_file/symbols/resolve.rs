use crate::per_file::ast_normalization::InModule;
use crate::per_file::symbols::resolve::control::{
    Decision, Reject, Rejects, SearchResult, WalkDownController, walk_down, walk_up,
};
use crate::per_file::symbols::resolve::types::{
    Opaque, Passthrough, SymbolIsNotType, UnexpectedUnresolvedReference, UnresolvedReference,
};
use crate::per_file::symbols::{SymbolKind, SymbolTable};
use crate::per_file::utils::{IterableSystemParam, ParIterableSystem, StaticQuery};
use crate::utils::TryReport;
use kodept_ast::Str;
use kodept_ast::prelude::{ASTNode, Erase, NodeId, Property};
use kodept_ast::properties::{HasProperty, Lexeme, Name, RequireProperty, SourceSpan};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::experimental::{Buffer, NodeModification};
use kodept_ast_nodes::{
    Module, Path, ResolvedType, ResolvedTypeAnnotation, TypeAnnotation, UnresolvedType,
    UserFunction, UserType, Value, Variable,
};
use kodept_core::code_point::Span;
use kodept_core::either::Either;
use kodept_core::structure::{Located, SpanBounds};
use kodept_ecs::component::Component;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::hierarchy::{ChildOf, Children};
use kodept_ecs::lifecycle::Add;
use kodept_ecs::query::{Has, Or, QueryFilter, With};
use kodept_ecs::system::{Commands, On, Query, Res, SystemParam};
use kodept_frontend::engine::reporter::Reporter;
use std::iter::Peekable;
use std::marker::PhantomData;
pub(crate) use types::ResolvedTo;

mod types {
    use crate::per_file::symbols::SymbolKind;
    use kodept_ast::prelude::NodeId;
    use kodept_ast::properties::{NodeProperty, RequireProperty};
    use kodept_ast_nodes::Value;
    use kodept_core::code_point::Span;
    use kodept_ecs::component::Component;
    use kodept_ecs::exported::bevy_ecs;
    use kodept_report::prelude::{Diagnostic, Severity};
    use kodept_report::traits::IntoMessage;
    use kodept_report_macros::IntoMessage;
    use std::borrow::Cow;
    use std::fmt::Display;

    #[derive(Debug, Component)]
    #[component(immutable)]
    pub(crate) struct ResolvedTo {
        pub referral: NodeId,
        pub kind: SymbolKind,
    }

    impl NodeProperty for ResolvedTo {}
    impl RequireProperty<ResolvedTo> for Value {}

    #[derive(Debug, Component)]
    #[component(immutable, storage = "SparseSet")]
    /// Marks nodes that inhibits resolution until node with [`Opaque`] component is found
    pub(super) struct Passthrough;
    #[derive(Debug, Component)]
    #[component(immutable, storage = "SparseSet")]
    /// Marks nodes that enabled resolution back
    pub(super) struct Opaque;

    #[derive(Debug)]
    pub(super) struct UnresolvedReference<'a, I> {
        pub(super) name: &'a str,
        pub(super) span: Span,
        pub(super) note: I,
    }

    #[derive(Debug, IntoMessage)]
    #[severity("error")]
    #[message("Definition of `{}` expected to be a type, but it is a {}", self.name, self.actual_kind
    )]
    pub(super) struct SymbolIsNotType<'a> {
        #[secondary_label("is not a type")]
        pub(super) span: Span,
        pub(super) actual_kind: SymbolKind,
        pub(super) name: &'a str,
        #[primary_label("required by {}", self.description)]
        pub(super) ref_span: Span,
        pub(super) description: Cow<'static, str>,
    }

    #[derive(Debug, IntoMessage)]
    #[severity("bug")]
    #[message("Reference `{}` is not resolved still", self.id)]
    pub(super) struct UnexpectedUnresolvedReference {
        pub(super) id: NodeId,
        #[primary_label("expected this to be resolved")]
        pub(super) span: Span,
    }

    impl<I> IntoMessage for UnresolvedReference<'_, I>
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
}

mod control {
    use kodept_ast::prelude::{Erase, NodeId};
    use kodept_ast::properties::Name;
    use kodept_ecs::entity::Entity;
    use kodept_ecs::hierarchy::{ChildOf, Children};
    use kodept_ecs::query::QueryData;
    use kodept_ecs::system::Query;
    use std::cmp::Ordering;
    use std::collections::{HashSet, VecDeque};
    use std::fmt::{Display, Formatter};

    #[derive(Debug)]
    pub(super) enum Decision<A, R> {
        Next,
        Reject(R),
        Accept(A),
    }

    #[derive(Debug)]
    pub(super) enum SearchResult<A, R> {
        Rejected(R),
        Accepted(NodeId, A),
    }

    #[derive(Debug)]
    enum Layered<T> {
        Layer,
        Value(T),
    }

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub(super) enum Reject<'a> {
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
    pub(super) struct Rejects<'a>(HashSet<Reject<'a>>);

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

    pub(super) fn walk_up<'s, T, A, R, P>(
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

    pub(super) trait WalkDownController<T> {
        type Accept;
        type Reject;

        fn on_iteration(&mut self, item: T) -> Decision<Self::Accept, Self::Reject>;
        fn on_layer(&mut self);
    }

    pub(super) fn walk_down<'s, T, A, R, P>(
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
                        Decision::Accept(accept) => {
                            return SearchResult::Accepted(id.into(), accept);
                        }
                    }
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
    query: Query<NodeId, With<UserFunction>>,
    mut commands: Commands,
) {
    for id in query {
        commands.entity(id.entity()).insert(Passthrough);
    }
    commands.add_observer(add_on_add(|| Passthrough, PhantomData::<UserFunction>));
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

#[derive(SystemParam)]
pub(super) struct ResolveValues<'w, 's> {
    properties: Properties<'w, 's>,
}

impl ParIterableSystem for ResolveValues<'_, '_> {
    type Iterable = Query<
        'static,
        'static,
        (
            NodeId<Value>,
            &'static Value,
            Property<InModule>,
            Property<SourceSpan>,
        ),
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (value, &InModule(module_id), &SourceSpan(span)) = params;

        match resolve_ident_with_path(
            modification.id(),
            module_id,
            &value.path,
            &value.ident,
            self.properties,
        ) {
            SearchResult::Rejected(rejects) => {
                return Err(UnresolvedReference {
                    name: &value.ident,
                    span,
                    note: rejects,
                });
            }
            SearchResult::Accepted(_, (resolved_to, kind)) => {
                modification.add_property(ResolvedTo {
                    referral: resolved_to,
                    kind,
                })
            }
        };
        Ok(())
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

#[derive(SystemParam)]
pub(super) struct ResolveTypeIn<'w, 's, Parent: 'static> {
    all_spans: Query<'w, 's, &'static SourceSpan>,
    properties: Properties<'w, 's>,
    syntax: Res<'w, SyntaxResolver>,
    _phantom: PhantomData<Parent>,
}

impl ParIterableSystem for ResolveTypeIn<'_, '_, Variable> {
    type Iterable = StaticQuery<(
        NodeId<Variable>,
        Property<TypeAnnotation>,
        Property<InModule>,
        Property<Lexeme>,
        Property<SourceSpan>,
    )>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (annotation, &InModule(module_id), &Lexeme(lexeme), &SourceSpan(span)) = params;
        let return_type_span = self
            .syntax
            .try_get::<kodept_rlt::prelude::InitializedVariable>(lexeme)
            .ok()
            .and_then(|it| it.variable.assigned_type.as_ref())
            .map(|it| it.1.bounds())
            .unwrap_or(span);

        match resolve_type_annotation(modification.id(), module_id, annotation, self.properties) {
            Ok(annotation) => {
                modification
                    .remove_property::<TypeAnnotation>()
                    .add_property(annotation);
                Ok(())
            }
            Err(TypeResolutionError::Rejected(rejects, name)) => {
                Err(Either::Left(UnresolvedReference {
                    span: return_type_span,
                    name,
                    note: rejects,
                }))
            }
            Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                Err(Either::Right(SymbolIsNotType {
                    name,
                    actual_kind,
                    span: self
                        .all_spans
                        .get(resolved_to.entity())
                        .map(|it| it.0)
                        .unwrap_or_default(),
                    ref_span: return_type_span,
                    description: "a type annotation".into(),
                }))
            }
        }
    }
}

pub(super) struct StrictType;

impl<P: 'static> ParIterableSystem<&'static str> for ResolveTypeIn<'_, '_, P>
where
    P: ASTNode
        + RequireProperty<TypeAnnotation>
        + RequireProperty<InModule>
        + HasProperty<ResolvedTypeAnnotation>,
{
    type Iterable = StaticQuery<(
        NodeId<P>,
        Property<TypeAnnotation>,
        Property<InModule>,
        Property<Lexeme>,
        Property<SourceSpan>,
    )>;

    fn for_each_with_input<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
        input: &&'static str,
    ) -> impl TryReport {
        let (annotation, &InModule(module_id), &Lexeme(lexeme), &SourceSpan(span)) = params;
        let ref_span = self
            .syntax
            .try_get_unknown(lexeme)
            .map(|it| it.location())
            .map(Span::from)
            .unwrap_or_default();

        match resolve_type_annotation(modification.id(), module_id, annotation, self.properties) {
            Ok(annotation) => {
                modification
                    .remove_property::<TypeAnnotation>()
                    .add_property(annotation);
                Ok(())
            }
            Err(TypeResolutionError::Rejected(rejects, name)) => {
                Err(Either::Left(UnresolvedReference {
                    name,
                    span,
                    note: rejects,
                }))
            }
            Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                Err(Either::Right(SymbolIsNotType {
                    name,
                    actual_kind,
                    ref_span,
                    span: self
                        .all_spans
                        .get(resolved_to.entity())
                        .map(|it| it.0)
                        .unwrap_or_default(),
                    description: format!("{input}").into(),
                }))
            }
        }
    }
}

impl<P: 'static> ParIterableSystem<(StrictType, &'static str)> for ResolveTypeIn<'_, '_, P>
where
    P: ASTNode
        + RequireProperty<UnresolvedType>
        + RequireProperty<InModule>
        + HasProperty<ResolvedType>,
{
    type Iterable = StaticQuery<(
        NodeId<P>,
        Property<UnresolvedType>,
        Property<InModule>,
        Property<Lexeme>,
        Property<SourceSpan>,
    )>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (annotation, &InModule(module_id), &Lexeme(lexeme), &SourceSpan(span)) = params;
        let ref_span = self
            .syntax
            .try_get_unknown(lexeme)
            .map(|it| it.location())
            .map(Span::from)
            .unwrap_or_default();

        match resolve_type(modification.id(), module_id, annotation, self.properties) {
            Ok(annotation) => {
                modification
                    .remove_property::<UnresolvedType>()
                    .add_property(annotation);
                Ok(())
            }
            Err(TypeResolutionError::Rejected(rejects, name)) => {
                Err(Either::Left(UnresolvedReference {
                    name,
                    span,
                    note: rejects,
                }))
            }
            Err(TypeResolutionError::WrongKind(resolved_to, actual_kind, name)) => {
                Err(Either::Right(SymbolIsNotType {
                    name,
                    actual_kind,
                    ref_span,
                    span: self
                        .all_spans
                        .get(resolved_to.entity())
                        .map(|it| it.0)
                        .unwrap_or_default(),
                    description: "a return type".into(),
                }))
            }
        }
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
