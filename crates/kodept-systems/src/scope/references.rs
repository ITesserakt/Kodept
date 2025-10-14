use crate::prelude::ExtractSymbolsPass;
use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::{Scoped, Visibility};
use crate::symbol::table::SymbolTable;
use crate::symbol::{DeferRefResolution, RefToSymbol, SymbolDescription, SymbolKind};
use crate::utils::{wrap_system, Ctx, Disposable, Interaction};
use bevy_ecs::event::{EntityEvent};
use bevy_ecs::hierarchy::Children;
use bevy_ecs::prelude::{
    ChildOf, Commands, Entity, Has, IntoScheduleConfigs, Name, Populated, Query, SystemSet, With,
    Without,
};
use bevy_ecs::query::{Added, AnyOf};
use bevy_ecs::system::Res;
use kodept_ast::prelude::HierarchicalQuery;
use kodept_ast::properties::{Lexeme, SourceSpan};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::Str;
use kodept_ast_nodes::expression::BinExpr;
use kodept_ast_nodes::file::ModDecl;
use kodept_ast_nodes::properties::Rhs;
use kodept_ast_nodes::term::{Ref, ReferenceContext};
use kodept_ast_nodes::types::Ty;
use kodept_report::message::Diagnostic;
use kodept_report::prelude::Severity;
use kodept_report::traits::IntoSpannedReportMessage;
use std::iter::once;

#[derive(Debug, Clone, Hash, Eq, PartialEq, SystemSet)]
pub struct ReferenceResolverPass;

/// Event that happens when a corresponding symbol is found for either `Ref` or `Ty`.
///
/// *This event is EntityEvent and target entity is reference itself*
#[derive(Debug, EntityEvent)]
pub(crate) struct ResolvedEvent {
    /// Id of a scope that contains found symbol
    #[event_target]
    pub scope_id: Entity,
    pub visibility: Visibility,
    pub kind: SymbolKind,
}

#[derive(Debug)]
struct ReferenceNotResolvedError {
    ref_name: Str,
    ref_span: SourceSpan,
    resolved_scope_span: Option<SourceSpan>,
}

impl IntoSpannedReportMessage for ReferenceNotResolvedError {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Error)
            .with_message(format!("Cannot resolve reference `{}`", self.ref_name))
            .with_primary_label("not found in scope", self.ref_span)
    }
}

impl Interaction for ReferenceResolverPass {
    fn install(ctx: &mut Ctx) -> impl Disposable + use<> {
        let set = (
            defer_reference_resolution_in_accesses_system,
            wrap_system(Self::name(), system).in_set(ReferenceResolverPass),
        )
            .chain();
        
        ctx.register(set);
        // ctx.register(debug_resolved_refs_system);
        ctx.configure_sets((ExtractSymbolsPass, ReferenceResolverPass).chain());
    }
}

fn defer_reference_resolution_in_accesses_system(
    query: HierarchicalQuery<
        BinExpr,
        Ref,
        Rhs,
        (Without<DeferRefResolution>, Without<RefToSymbol>),
    >,
    mut commands: Commands,
) {
    for (_, child_id, parent, _) in query.iter() {
        commands
            .entity(child_id.entity())
            .insert_if(DeferRefResolution, || matches!(parent, BinExpr::Access));
    }
}

fn debug_resolved_refs_system(
    query: Populated<&Lexeme, Added<RefToSymbol>>,
    reporter: Reporter,
    points: Res<SyntaxResolver>,
) {
    for lexeme in query.iter() {
        reporter.report_ad_hoc(|| {
            Diagnostic::new(Severity::Note)
                .with_message("Reference resolved")
                .with_primary_label("resolved", points.get_span(lexeme.0))
        });
    }
}

const CANNOT_GET_SYMBOL_TABLE_FAILURE: &'static str = "Cannot get symbol table for given scope";
const CANNOT_GET_SCOPE_NAME_FAILURE: &'static str = "Cannot get name for given scope";
const CANNOT_GET_SUBSCOPE_FAILURE: &'static str = "Cannot get subscope for given scope";

fn resolve_ref_at(
    symbol_to_find: &mut SymbolDescription,
    current_scope_id: Entity,
    table: &SymbolTable,
) -> Option<RefToSymbol> {
    symbol_to_find.reset();
    loop {
        if let Some(_) = table.get(symbol_to_find) {
            return Some(RefToSymbol::new(current_scope_id, symbol_to_find));
        }
        if !symbol_to_find.iter_kinds() {
            return None;
        }
    }
}

fn resolve_ref_with_empty_local_context(
    symbol_to_find: &mut SymbolDescription,
    current_scope_id: Entity,
    scope_parents_query: Query<&ChildOf, With<Scope>>,
    symbol_tables_query: Query<&SymbolTable>,
) -> Option<RefToSymbol> {
    for scope_id in
        once(current_scope_id).chain(scope_parents_query.iter_ancestors(current_scope_id))
    {
        let table = symbol_tables_query
            .get(scope_id)
            .expect(CANNOT_GET_SYMBOL_TABLE_FAILURE);
        if let Some(result) = resolve_ref_at(symbol_to_find, scope_id, table) {
            return Some(result);
        }
    }
    None
}

fn resolve_ref_at_through_context(
    context: &ReferenceContext,
    symbol_to_find: &mut SymbolDescription,
    starting_scope_id: Entity,
    scope_children_query: Query<&Children, With<Scope>>,
    scope_names_query: Query<Option<&Name>, With<Scope>>,
    symbol_tables_query: Query<&SymbolTable>,
) -> Option<RefToSymbol> {
    let mut stack = Vec::with_capacity(context.items.len());
    stack.extend(
        scope_children_query
            .get(starting_scope_id)
            .iter()
            .flat_map(|it| it.iter())
            .map(|it| (*it, 0)),
    );

    let mut maybe_target_scope = None;
    while let Some((scope_id, index)) = stack.pop() {
        let scope_name = scope_names_query
            .get(scope_id)
            .expect(CANNOT_GET_SCOPE_NAME_FAILURE);
        let part = &context.items[index];
        if scope_name.is_none_or(|it| it.as_str() != part) {
            continue;
        }

        if index == context.items.len() - 1 {
            maybe_target_scope = Some(scope_id);
            break;
        }

        stack.extend(
            scope_children_query
                .get(scope_id)
                .iter()
                .flat_map(|it| it.iter())
                .map(|it| (*it, index + 1))
                .rev(),
        );
    }

    let target_scope = maybe_target_scope?;
    let table = symbol_tables_query
        .get(target_scope)
        .expect(CANNOT_GET_SYMBOL_TABLE_FAILURE);
    resolve_ref_at(symbol_to_find, target_scope, table)
}

fn system(
    references: Populated<
        (Entity, AnyOf<(&Ref, &Ty)>, &Scoped, &SourceSpan),
        (Without<RefToSymbol>, Without<DeferRefResolution>),
    >,
    scope_parents: Query<&ChildOf, With<Scope>>,
    scope_children: Query<&Children, With<Scope>>,
    scope_names: Query<Option<&Name>, With<Scope>>,
    symbol_tables: Query<&SymbolTable>,
    scopes: Query<&Scope>,
    modules: Query<Has<ModDecl>>,
    mut commands: Commands,
) {
    for (id, reference, scoped, _) in references.iter() {
        let (context, mut symbol_description) = match reference {
            (None, None) => unreachable!("It's guaranteed to have either `ref` or `ty`"),
            (None, Some(ty)) => (
                &ty.context,
                SymbolDescription::new(Name::new(ty.ident.clone()), SymbolKind::Type),
            ),
            (Some(r), None) => (
                &r.context,
                SymbolDescription::new(Name::new(r.ident.clone()), SymbolKind::Variable),
            ),
            (Some(_), Some(_)) => unreachable!("Both of `ref` and `ty` cannot be on one entity"),
        };

        // There are four different situations...
        let result = if context.is_empty_local_context() {
            // Ascend by scopes until we find the appropriate symbol
            resolve_ref_with_empty_local_context(
                &mut symbol_description,
                scoped.0,
                scope_parents,
                symbol_tables,
            )
        } else if context.is_empty_global_context() {
            let root_scope_id = scope_parents.root_ancestor::<ChildOf>(scoped.0);
            let table = symbol_tables
                .get(root_scope_id)
                .expect(CANNOT_GET_SYMBOL_TABLE_FAILURE);
            resolve_ref_at(&mut symbol_description, root_scope_id, table)
        } else if context.global {
            // Take root scope and descend deeper through context
            let root_scope = scope_parents.root_ancestor::<ChildOf>(scoped.0);
            resolve_ref_at_through_context(
                &context,
                &mut symbol_description,
                root_scope,
                scope_children,
                scope_names,
                symbol_tables,
            )
        } else {
            // Find scope of the current module. Later, we can add check for imports
            // Than descend deeper through context
            let mod_scope = once(scoped.0)
                .chain(scope_parents.iter_ancestors(scoped.0))
                .find(|it| {
                    let scope = scopes.get(*it).expect("Cannot get given scope");
                    modules.get(scope.start_from.entity()).unwrap_or(false)
                });
            match mod_scope {
                Some(mod_scope) => resolve_ref_at_through_context(
                    &context,
                    &mut symbol_description,
                    mod_scope,
                    scope_children,
                    scope_names,
                    symbol_tables,
                ),
                // Some modules are still not scoped, suppress fail and skip
                None => continue,
            }
        };

        if let Some(ref_to_symbol) = result {
            commands.entity(id).insert(ref_to_symbol);
        }
    }
}
