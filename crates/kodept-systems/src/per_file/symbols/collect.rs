use crate::per_file::symbols::{
    ComponentIdForMapper, RefMapper, SymbolKind, SymbolName, SymbolTable,
};
use crate::source::collection::Reporter;
use kodept_ast::Str;
use kodept_ast::prelude::{ASTNode, Erase, HierarchicalQuery, NodeId, NodeQueryData};
use kodept_ast::properties::{HasProperty, Lexeme, Name, SourceSpan};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::children::{Family, MembersOf};
use kodept_ast::syntax_tree::experimental::NodeModification;
use kodept_ast_nodes::{
    CtorName, Declaration, Module, NormalizedBlock, ResolvedTypeAnnotation, Statement,
    TypeAnnotation, UserFunction, UserType, ValueCtor, Variable, VariableName,
};
use kodept_core::code_point::Span;
use kodept_core::structure::Located;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::{AnyOf, Has, QueryItem, ROQueryItem, ReadOnlyQueryData, With};
use kodept_ecs::system::{Commands, Query, Res, StaticSystemParam, SystemParam, SystemParamItem};
use kodept_report::message::{Diagnostic, Severity};
use kodept_report_macros::Report;
use kodept_rlt::traversal::ErasedNodeBorrow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Report)]
#[severity("error")]
#[message("Element `{}` has already been defined", self.name)]
#[note("Name of {} clashes with another {}", self.current_kind, self.previous_kind)]
struct DuplicateSymbol<'a> {
    name: &'a str,
    current_kind: SymbolKind,
    previous_kind: SymbolKind,
    #[primary_label]
    current_symbol_span: Span,
    #[secondary_label("previous declaration")]
    previous_symbol_span: Span,
    #[secondary_label("in scope")]
    scope_span: Span,
}

#[derive(SystemParam)]
pub(super) struct SymbolsCollector<'w, 's, T: SystemParam + 'static> {
    inner: StaticSystemParam<'w, 's, T>,
    spans: Query<'w, 's, (&'static SourceSpan, &'static Lexeme)>,
    syntax: Res<'w, SyntaxResolver>,
    reporter: Reporter<'w, 's>,
    commands: Commands<'w, 's>,
}

pub(super) struct SymbolRegistrator<'a, 'w, 's> {
    table: &'a mut SymbolTable,
    reporter: &'a mut Reporter<'w, 's>,
    spans: &'a Query<'w, 's, (&'static SourceSpan, &'static Lexeme)>,
    syntax: &'a SyntaxResolver,
    scope_span: Span,
    custom_into_span: fn(ErasedNodeBorrow) -> Option<Span>,
}

impl SymbolRegistrator<'_, '_, '_> {
    fn register(&mut self, name: impl Into<SymbolName>, kind: SymbolKind, id: impl Erase) {
        let id = id.erase();
        let name = name.into();
        if let Err((previous_kind, old_id)) = self.table.insert(name.clone(), kind.clone(), id) {
            let span_for = |id| {
                self.spans
                    .get(id)
                    .map(|it| (it.0, self.syntax.try_get_unknown(it.1.0)))
                    .ok()
                    .and_then(|it| Some((it.0, it.1?)))
                    .map(|it| (self.custom_into_span)(it.1).unwrap_or(it.0.0))
                    .unwrap_or_default()
            };

            self.reporter.report(DuplicateSymbol {
                name: name.0.as_ref(),
                scope_span: self.scope_span,
                previous_kind: previous_kind.clone(),
                current_kind: kind.clone(),
                current_symbol_span: span_for(id.entity()),
                previous_symbol_span: span_for(old_id.entity()),
            });
        }
    }
}

pub(super) trait CollectSymbols<Tag>
where
    Self: ASTNode + Sized + HasProperty<SymbolTable> + Family<Tag>,
{
    type ParentFetch: NodeQueryData<Self>;
    type ChildFetch: ReadOnlyQueryData;
    type Params: SystemParam + 'static;

    #[inline]
    fn get_scope_span(lexeme: ErasedNodeBorrow) -> Option<Span> {
        _ = lexeme;
        None
    }

    fn system(
        mut state: SymbolsCollector<(
            HierarchicalQuery<
                Self,
                Tag,
                (
                    Self::ParentFetch,
                    Option<&mut SymbolTable>,
                    &SourceSpan,
                    &Lexeme,
                ),
                (&SourceSpan, Self::ChildFetch),
            >,
            Self::Params,
        )>,
    ) {
        let (mut query, mut params) = state.inner.into_inner();
        for (id, (additional, table, span, lexeme), children) in query.iter_by_layers() {
            match table {
                None => {
                    let mut modification = NodeModification::new(state.commands.reborrow(), id);
                    let mut table = SymbolTable::default();
                    let registrator = SymbolRegistrator {
                        reporter: &mut state.reporter,
                        table: &mut table,
                        spans: &state.spans,
                        syntax: &*state.syntax,
                        scope_span: state
                            .syntax
                            .try_get_unknown(lexeme.0)
                            .and_then(|it| Self::get_scope_span(it))
                            .unwrap_or(span.0),
                        custom_into_span: |lexeme| Some(lexeme.location().into()),
                    };
                    Self::each(
                        id,
                        additional,
                        children.iter().map(|it| (it.0, it.1.1)),
                        &mut params,
                        registrator,
                    );
                    modification.add_property(table);
                }
                Some(mut table) => {
                    let registrator = SymbolRegistrator {
                        reporter: &mut state.reporter,
                        table: &mut *table,
                        spans: &state.spans,
                        syntax: &*state.syntax,
                        scope_span: state
                            .syntax
                            .try_get_unknown(lexeme.0)
                            .and_then(|it| Self::get_scope_span(it))
                            .unwrap_or(span.0),
                        custom_into_span: |lexeme| Some(lexeme.location().into()),
                    };
                    Self::each(
                        id,
                        additional,
                        children.iter().map(|it| (it.0, it.1.1)),
                        &mut params,
                        registrator,
                    );
                }
            }
        }
    }

    fn each<'a>(
        id: NodeId<Self>,
        additional: QueryItem<Self::ParentFetch>,
        children: impl Iterator<Item = (NodeId, ROQueryItem<'a, 'a, Self::ChildFetch>)>,
        params: &mut SystemParamItem<Self::Params>,
        registrator: SymbolRegistrator,
    );
}

impl CollectSymbols<Declaration> for Module {
    type ParentFetch = ();
    type ChildFetch = (&'static Archetype, &'static Name);
    type Params = MembersOf<Module, Declaration, ComponentIdForMapper<'static>>;

    fn each<'a>(
        _: NodeId<Self>,
        _: QueryItem<Self::ParentFetch>,
        children: impl Iterator<Item = (NodeId, (&'a Archetype, &'a Name))>,
        params: &mut SystemParamItem<Self::Params>,
        mut registrator: SymbolRegistrator,
    ) {
        for (child_id, (archetype, name)) in children {
            if archetype.contains(params.0.get()) {
                registrator.register(name, SymbolKind::Type, child_id);
            } else if archetype.contains(params.1.get()) {
                registrator.register(name, SymbolKind::Type, child_id);
            } else {
                registrator.register(name, SymbolKind::Function, child_id);
            }
        }
    }
}

impl CollectSymbols<Declaration> for UserType {
    type ParentFetch = ();
    type ChildFetch = &'static Name;
    type Params = ();

    fn each<'a>(
        _: NodeId<Self>,
        _: QueryItem<Self::ParentFetch>,
        children: impl Iterator<Item = (NodeId, ROQueryItem<'a, 'a, Self::ChildFetch>)>,
        _: &mut SystemParamItem<Self::Params>,
        mut registrator: SymbolRegistrator,
    ) {
        for (child_id, name) in children {
            registrator.register(name, SymbolKind::Function, child_id);
        }
    }
}

impl CollectSymbols<()> for UserType {
    type ParentFetch = &'static Name;
    type ChildFetch = AnyOf<MembersOf<UserType, (), RefMapper<'static>>>;
    type Params = ();

    fn each<'a>(
        _: NodeId<Self>,
        type_name: QueryItem<Self::ParentFetch>,
        children: impl Iterator<Item = (NodeId, ROQueryItem<'a, 'a, Self::ChildFetch>)>,
        _: &mut SystemParamItem<Self::Params>,
        mut registrator: SymbolRegistrator,
    ) {
        for (child_id, ctor) in children {
            let name = match ctor {
                (
                    Some(ValueCtor {
                        name: CtorName::Inline,
                        ..
                    }),
                    None,
                )
                | (
                    None,
                    Some(ValueCtor {
                        name: CtorName::Inline,
                        ..
                    }),
                ) => SymbolName::from(type_name),
                (
                    Some(ValueCtor {
                        name: CtorName::Explicit(name),
                        ..
                    }),
                    None,
                )
                | (
                    None,
                    Some(ValueCtor {
                        name: CtorName::Explicit(name),
                        ..
                    }),
                ) => SymbolName::from(name),
                _ => unreachable!(),
            };
            registrator.register(name, SymbolKind::Constructor, child_id);
        }
    }
}

impl CollectSymbols<Statement> for NormalizedBlock {
    type ParentFetch = ();
    type ChildFetch = (
        Option<&'static Name>,
        Has<UserFunction<TypeAnnotation>>,
        Has<UserFunction<ResolvedTypeAnnotation>>,
        Option<&'static Variable<TypeAnnotation>>,
        Option<&'static Variable<ResolvedTypeAnnotation>>,
    );
    type Params = ();

    fn each<'a>(
        _: NodeId<Self>,
        _: QueryItem<Self::ParentFetch>,
        children: impl Iterator<Item = (NodeId, ROQueryItem<'a, 'a, Self::ChildFetch>)>,
        _: &mut SystemParamItem<Self::Params>,
        mut registrator: SymbolRegistrator,
    ) {
        for (child_id, (name, is_function, is_resolved_function, variable, resolved_variable)) in
            children
        {
            if let Some(name) = name
                && (is_function || is_resolved_function)
            {
                registrator.register(name, SymbolKind::Function, child_id);
            } else if let Some(Variable {
                name: VariableName::Name(name),
                ..
            }) = variable
            {
                registrator.register(name, SymbolKind::Variable, child_id);
            } else if let Some(Variable {
                name: VariableName::Name(name),
                ..
            }) = resolved_variable
            {
                registrator.register(name, SymbolKind::Variable, child_id);
            }
        }
    }
}

pub(super) fn collect_params_on<T, U>(
    get_params: impl Fn(&T) -> &[U],
    get_param_name: impl Fn(&U) -> &Str,
) -> impl FnMut(SymbolsCollector<Query<(NodeId<T>, &T, &SourceSpan)>>)
where
    T: ASTNode + HasProperty<SymbolTable>,
{
    move |mut state| {
        for (id, ctor, span) in state.inner.into_inner() {
            let mut modification = NodeModification::new(state.commands.reborrow(), id);
            let mut table = SymbolTable::default();
            let mut registrator = SymbolRegistrator {
                table: &mut table,
                syntax: &*state.syntax,
                spans: &state.spans,
                reporter: &mut state.reporter,
                custom_into_span: |lexeme| Some(lexeme.location().into()),
                scope_span: span.0,
            };

            for (index, param) in get_params(ctor).iter().enumerate() {
                registrator.register(get_param_name(param), SymbolKind::Parameter(index), id);
            }

            modification.add_property(table);
        }
    }
}

pub(super) fn check_module_names(modules: Query<(&Name, &SourceSpan), With<Module>>, mut reporter: Reporter) {
    let mut set = HashMap::new();
    for (name, span) in modules {
        match set.entry(name.as_ref()) {
            Entry::Occupied(entry) => {
                let old_span = entry.get();

                reporter.report_ad_hoc(|| {
                    Diagnostic::new(Severity::Error)
                        .with_message(format!("Module `{}` has already been defined", entry.key()))
                        .with_primary_label("", span.0)
                        .with_secondary_label("previous definition", *old_span)
                });
            }
            Entry::Vacant(entry) => {
                entry.insert(span.0);
            }
        }
    }
}
