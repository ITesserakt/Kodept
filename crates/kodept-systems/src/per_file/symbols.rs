use crate::source::collection::Reporter;
use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::ComponentIdFor;
use bevy_ecs::prelude::{AnyOf, Commands, Has, Name, Ref, Res};
use bevy_ecs::query::{QueryItem, ROQueryItem, ReadOnlyQueryData};
use bevy_ecs::system::{Query, StaticSystemParam, SystemParam, SystemParamItem};
use kodept_ast::export::Component;
use kodept_ast::prelude::{ASTNode, Erase, HierarchicalQuery, NodeId, NodeQueryData};
use kodept_ast::properties::{HasProperty, Lexeme, NodeProperty, RequireProperty, SourceSpan};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::children::{Family, MembersOf, Wrapper};
use kodept_ast::syntax_tree::experimental::NodeModification;
use kodept_ast_nodes::{
    CtorName, Declaration, Module, NormalizedBlock, Param, ResolvedTypeAnnotation, Statement,
    TypeAnnotation, TypeRef, UnresolvedName, UserFunction, UserType, Value, ValueCtor, Variable,
    VariableName,
};
use kodept_core::code_point::Span;
use kodept_core::structure::Located;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;
use kodept_rlt::traversal::ErasedNodeBorrow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

struct ComponentIdForMapper<'a>(PhantomData<&'a ()>);
impl<'a> Wrapper for ComponentIdForMapper<'a> {
    type Wrapped<T: Component> = ComponentIdFor<'a, T>;
}
struct RefMapper<'a>(PhantomData<&'a ()>);
impl<'a> Wrapper for RefMapper<'a> {
    type Wrapped<T: Component> = &'a T;
}

define_phase! {
    pub phase ReferenceResolutionPhase[ReferenceResolutionPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(NormalizedBlock::system);
        engine.add_systems(UserFunction::<TypeAnnotation>::system);
        engine.add_systems(UserFunction::<ResolvedTypeAnnotation>::system);
        engine.add_systems(<UserType as CollectSymbols<Declaration>>::system);
        engine.add_systems(<UserType as CollectSymbols<()>>::system);
        engine.add_systems(Module::system);
    }
}

#[derive(Debug, Component, Default)]
#[component(immutable)]
struct SymbolTable {
    order: Vec<(SymbolKind, NodeId)>,
    names: HashMap<Name, usize>,
}
impl NodeProperty for SymbolTable {}
impl RequireProperty<SymbolTable> for Module {}
impl RequireProperty<SymbolTable> for UserType {}
impl<T: TypeRef<false>> RequireProperty<SymbolTable> for UserFunction<T> {}
impl RequireProperty<SymbolTable> for NormalizedBlock {}

#[derive(Debug, Clone)]
enum SymbolKind {
    Type,
    Function,
    Constructor,
    Parameter(usize),
    Variable,
}

#[derive(Debug, Report)]
#[severity("error")]
#[message("Element with name `{}` already defined", self.name)]
#[note("Name of {} clashes with other {}", self.current_kind, self.previous_kind)]
struct DuplicateSymbol {
    name: Name,
    current_kind: SymbolKind,
    previous_kind: SymbolKind,
    #[primary_label]
    current_symbol_span: Span,
    #[secondary_label("previous declaration")]
    previous_symbol_span: Span,
    #[secondary_label("in scope")]
    scope_span: Span,
}

impl Display for SymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Type => write!(f, "type"),
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Constructor => write!(f, "constructor"),
            SymbolKind::Parameter(_) => write!(f, "parameter"),
            SymbolKind::Variable => write!(f, "variable"),
        }
    }
}

impl SymbolTable {
    fn insert(
        &mut self,
        name: Name,
        kind: SymbolKind,
        id: impl Erase,
    ) -> Result<(), (&SymbolKind, NodeId)> {
        let index = self.order.len();
        self.order.push((kind, id.erase()));
        if let Some(registered_index) = self.names.insert(name, index) {
            let value = &self.order[registered_index];
            return Err((&value.0, value.1));
        }
        Ok(())
    }
}

#[derive(SystemParam)]
struct SymbolsCollector<'w, 's, T: SystemParam + 'static> {
    inner: StaticSystemParam<'w, 's, T>,
    spans: Query<'w, 's, (&'static SourceSpan, &'static Lexeme)>,
    syntax: Res<'w, SyntaxResolver>,
    reporter: Reporter<'w, 's>,
    commands: Commands<'w, 's>,
}

struct SymbolRegistrator<'a, 'w, 's> {
    table: &'a mut SymbolTable,
    reporter: &'a mut Reporter<'w, 's>,
    spans: &'a Query<'w, 's, (&'static SourceSpan, &'static Lexeme)>,
    syntax: &'a SyntaxResolver,
    scope_span: Span,
    custom_into_span: fn(ErasedNodeBorrow) -> Option<Span>,
}

trait CollectSymbols<Tag>
where
    Self: ASTNode + Sized + HasProperty<SymbolTable> + Family<Tag>,
{
    type ParentFetch: NodeQueryData<Self, ReadOnly = Self::ParentFetch>;
    type ChildFetch: ReadOnlyQueryData;
    type Params: SystemParam + 'static;

    #[inline]
    fn get_scope_span(lexeme: ErasedNodeBorrow) -> Option<Span> {
        _ = lexeme;
        None
    }

    fn system(
        mut state: SymbolsCollector<(
            HierarchicalQuery<Self, Tag, Self::ParentFetch, (&SourceSpan, Self::ChildFetch)>,
            Self::Params,
        )>,
    ) {
        let (query, mut params) = state.inner.into_inner();
        for (id, additional, children) in query.iter_by_layers() {
            let mut modification = NodeModification::new(state.commands.reborrow(), id);
            let mut table = SymbolTable::default();
            let Ok((scope_span, lexeme)) = state.spans.get(id.entity()) else {
                continue;
            };

            Self::each(
                id,
                additional,
                children.iter().map(|it| (it.0, it.1.1)),
                &mut params,
                SymbolRegistrator {
                    reporter: &mut state.reporter,
                    table: &mut table,
                    spans: &state.spans,
                    syntax: &*state.syntax,
                    scope_span: state
                        .syntax
                        .try_get_unknown(lexeme.0)
                        .and_then(|it| Self::get_scope_span(it))
                        .unwrap_or(scope_span.0),
                    custom_into_span: |lexeme| Some(lexeme.location().into()),
                },
            );

            modification.add_property(table);
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

impl SymbolRegistrator<'_, '_, '_> {
    fn register(&mut self, name: Name, kind: SymbolKind, id: impl Erase) {
        let id = id.erase();
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
                name,
                scope_span: self.scope_span,
                previous_kind: previous_kind.clone(),
                current_kind: kind.clone(),
                current_symbol_span: span_for(id.entity()),
                previous_symbol_span: span_for(old_id.entity()),
            });
        }
    }

    fn set_default_into_span(&mut self) {
        self.custom_into_span = |lexeme| Some(lexeme.location().into());
    }
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
                registrator.register(name.clone(), SymbolKind::Type, child_id);
            } else if archetype.contains(params.1.get()) {
                registrator.register(name.clone(), SymbolKind::Type, child_id);
            } else {
                registrator.register(name.clone(), SymbolKind::Function, child_id);
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
            registrator.register(name.clone(), SymbolKind::Function, child_id);
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
                ) => type_name.clone(),
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
                ) => Name::new(name.clone()),
                _ => unreachable!(),
            };
            registrator.register(name, SymbolKind::Constructor, child_id);
        }
    }
}

impl<T: TypeRef<false>> CollectSymbols<()> for UserFunction<T> {
    type ParentFetch = Ref<'static, Self>;
    type ChildFetch = ();
    type Params = ();

    fn each<'a>(
        id: NodeId<Self>,
        value: QueryItem<Self::ParentFetch>,
        _: impl Iterator<Item = (NodeId, ROQueryItem<'a, 'a, Self::ChildFetch>)>,
        _: &mut SystemParamItem<Self::Params>,
        mut registrator: SymbolRegistrator,
    ) {
        for (index, param) in value.params.iter().enumerate() {
            match param {
                Param::Positional {
                    name: Some(name), ..
                } => {
                    let name = Name::new(name.clone());
                    registrator.register(name, SymbolKind::Parameter(index), id);
                }
                Param::Positional { name: None, .. } => {
                    let name = Name::new(index.to_string());
                    registrator.register(name, SymbolKind::Parameter(index), id);
                }
                Param::Named { name, .. } => {
                    let name = Name::new(name.clone());
                    registrator.register(name, SymbolKind::Parameter(index), id);
                }
            }
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
                registrator.register(name.clone(), SymbolKind::Function, child_id);
            } else if let Some(Variable {
                name: VariableName::Name(name),
                ..
            }) = variable
            {
                let name = Name::new(name.clone());
                registrator.register(name, SymbolKind::Variable, child_id);
            } else if let Some(Variable {
                name: VariableName::Name(name),
                ..
            }) = resolved_variable
            {
                let name = Name::new(name.clone());
                registrator.register(name, SymbolKind::Variable, child_id);
            }
        }
    }
}

fn resolve_values(values: Query<(NodeId<Value<UnresolvedName>>, &Value<UnresolvedName>)>) {}
