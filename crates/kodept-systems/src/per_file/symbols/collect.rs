use crate::per_file::symbols::{SymbolKind, SymbolTable};
use crate::per_file::utils::{IterableSystemParam, ParIterableSystem};
use crate::source::collection::Reporter;
use crate::utils::TryReport;
use kodept_ast::prelude::{ASTNode, MutProperty, NarrowHierarchicalQuery, Property};
use kodept_ast::properties::{HasProperty, Name, RequireProperty, SourceSpan};
use kodept_ast::syntax_tree::children::HasChild;
use kodept_ast::syntax_tree::experimental::{Buffer, NodeModification};
use kodept_ast_nodes::{CtorName, Module, ValueCtor, Variable, VariableName};
use kodept_core::code_point::Span;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::query::With;
use kodept_ecs::system::{Query, SystemParam};
use kodept_report::message::{Diagnostic, Severity};
use kodept_report_macros::Report;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

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
pub(super) struct CollectSymbolsIn<'w, 's, Parent: 'static, Tag: 'static, Child: 'static> {
    spans: Query<'w, 's, Property<SourceSpan>>,
    _phantom: PhantomData<(Parent, Tag, Child)>,
}

#[derive(Debug)]
enum Unique<'a, T> {
    Ref(&'a mut T),
    Owned(T),
}

impl<T: Default> Default for Unique<'_, T> {
    fn default() -> Self {
        Self::Owned(T::default())
    }
}

impl<'a, T> Deref for Unique<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Unique::Ref(x) => x,
            Unique::Owned(x) => x,
        }
    }
}

impl<'a, T> DerefMut for Unique<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Unique::Ref(x) => x,
            Unique::Owned(x) => x,
        }
    }
}

type Iterable<P, C, T, D> =
    NarrowHierarchicalQuery<'static, 'static, P, C, T, Option<MutProperty<SymbolTable>>, D>;

impl<P: 'static, T: 'static, C: 'static> ParIterableSystem<SymbolKind>
    for CollectSymbolsIn<'_, '_, P, T, C>
where
    P: HasChild<C, T> + HasProperty<SymbolTable>,
    C: ASTNode + RequireProperty<Name>,
{
    type Iterable = Iterable<P, C, T, Property<Name>>;

    fn for_each_with_input<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
        kind: &SymbolKind,
    ) -> impl TryReport {
        let (mut table, params) = params;
        let mut table = table.as_deref_mut().map_or(Unique::default(), Unique::Ref);

        for (child_id, name) in params {
            match table.insert(name, *kind, child_id) {
                Ok(()) => {}
                Err((old_kind, old_id)) => {
                    let span_of = |id| self.spans.get(id).map(|it| it.0).unwrap_or_default();
                    return Err(DuplicateSymbol {
                        name: name.as_str(),
                        current_kind: *kind,
                        previous_kind: old_kind,
                        current_symbol_span: span_of(child_id.entity()),
                        previous_symbol_span: span_of(old_id.entity()),
                        scope_span: span_of(modification.id().entity()),
                    });
                }
            }
        }

        if let Unique::Owned(table) = table {
            modification.add_property(table);
        }
        Ok(())
    }
}

impl<P: 'static, T: 'static> ParIterableSystem for CollectSymbolsIn<'_, '_, P, T, Variable>
where
    P: HasChild<Variable, T> + HasProperty<SymbolTable>,
{
    type Iterable = Iterable<P, Variable, T, &'static Variable>;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let (mut table, params) = params;
        let mut table = table.as_deref_mut().map_or(Unique::default(), Unique::Ref);

        for (child_id, variable) in params {
            let VariableName::Name(name) = &variable.name else {
                continue;
            };

            match table.insert(name, SymbolKind::Variable, child_id) {
                Ok(()) => {}
                Err((old_kind, old_id)) => {
                    let span_of = |id| self.spans.get(id).map(|it| it.0).unwrap_or_default();
                    return Err(DuplicateSymbol {
                        name,
                        current_kind: SymbolKind::Variable,
                        previous_kind: old_kind,
                        current_symbol_span: span_of(child_id.entity()),
                        previous_symbol_span: span_of(old_id.entity()),
                        scope_span: span_of(modification.id().entity()),
                    });
                }
            }
        }

        if let Unique::Owned(table) = table {
            modification.add_property(table);
        }
        Ok(())
    }
}

impl<P: 'static, T: 'static> ParIterableSystem for CollectSymbolsIn<'_, '_, P, T, ValueCtor>
where
    P: HasChild<ValueCtor, T> + HasProperty<SymbolTable> + RequireProperty<Name>,
{
    type Iterable = NarrowHierarchicalQuery<
        'static,
        'static,
        P,
        ValueCtor,
        T,
        (Option<MutProperty<SymbolTable>>, Property<Name>),
        &'static ValueCtor,
    >;

    fn for_each<B: Buffer>(
        &self,
        mut modification: NodeModification<<Self::Iterable as IterableSystemParam>::Node, B>,
        params: <Self::Iterable as IterableSystemParam>::Target<'_, '_>,
    ) -> impl TryReport {
        let ((mut table, parent_name), params) = params;
        let mut table = table.as_deref_mut().map_or(Unique::default(), Unique::Ref);

        for (child_id, ctor) in params {
            let name = match &ctor.name {
                CtorName::Inline => parent_name.as_ref(),
                CtorName::Explicit(name) => name,
            };

            match table.insert(name, SymbolKind::Constructor, child_id) {
                Ok(()) => {}
                Err((old_kind, old_id)) => {
                    let span_of = |id| self.spans.get(id).map(|it| it.0).unwrap_or_default();
                    return Err(DuplicateSymbol {
                        name,
                        current_kind: SymbolKind::Constructor,
                        previous_kind: old_kind,
                        current_symbol_span: span_of(child_id.entity()),
                        previous_symbol_span: span_of(old_id.entity()),
                        scope_span: span_of(modification.id().entity()),
                    });
                }
            }
        }

        if let Unique::Owned(table) = table {
            modification.add_property(table);
        }
        Ok(())
    }
}

pub(super) fn check_module_names(
    modules: Query<(&Name, &SourceSpan), With<Module>>,
    mut reporter: Reporter,
) {
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
