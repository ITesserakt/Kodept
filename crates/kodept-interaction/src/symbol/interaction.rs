use crate::report::Reporter;
use crate::scope::Scoped;
use crate::scope::storage::Scope;
use crate::symbol::SymbolKind::{self, Function, Parameter, Type, Variable};
use crate::symbol::table::SymbolTable;
use crate::symbol::{Declaration, RefToSymbol, SymbolData, SymbolDescription};
use crate::utils::{Ctx, Disposable, Interaction, wrap_system};
use bevy_ecs::prelude::{
    Added, AnyOf, Commands, Insert, On, Query, SystemSet, Without, any_match_filter,
};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::schedule::IntoScheduleConfigs;
use kodept_ast::define_union;
use kodept_ast::prelude::AnyNodeRef;
use kodept_ast::properties::{Name, Node, SourceSpan};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::consts::Const;
use kodept_ast_nodes::term::Ref;
use kodept_ast_nodes::top_level::EnumConst;
use kodept_ast_nodes::types::{NonTyParam, Ty, TyParam};
use kodept_core::code_point::Span;
use kodept_report::message::{Diagnostic, Severity};
use kodept_report::traits::IntoSpannedReportMessage;
use std::borrow::Cow;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemSet)]
pub struct ExtractSymbolsPass;

define_union!(enum SymbolUnion[SymbolUnionItem, SymbolUnionFilter] {
    Const | VarDecl | EnumConst | TyParam | NonTyParam
});

#[derive(Debug)]
pub struct DuplicatedSymbolError {
    bound_name: Name,
    scope_name: Option<Name>,
    scope_start: Span,
    current_def: Span,
    previous_def: Span,
}

impl IntoSpannedReportMessage for DuplicatedSymbolError {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        let scope_name_message = match self.scope_name {
            Some(name) => Cow::Owned(format!("in scope `{name}`")),
            None => "in scope".into(),
        };
        Diagnostic::new(Severity::Error)
            .with_message(format!(
                "Element with name `{}` already defined",
                self.bound_name
            ))
            .with_primary_label("", self.current_def)
            .with_secondary_label("previous declaration", self.previous_def)
            .with_secondary_label(scope_name_message, self.scope_start)
    }
}

impl Interaction for ExtractSymbolsPass {
    fn install(ctx: &mut Ctx) -> impl Disposable + use<> {
        ctx.register(
            wrap_system(Self::name(), Self::system)
                .run_if(any_match_filter::<(SymbolUnionFilter, Added<Scoped>)>)
                .in_set(ExtractSymbolsPass),
        );
        ctx.immediate_exclusive(|w| {
            w.add_observer(Self::sustain_declaration_links);
        });
    }
}

impl ExtractSymbolsPass {
    fn sustain_declaration_links(
        target: On<Insert, RefToSymbol>,
        refs: Query<(&RefToSymbol, AnyOf<(&Ref, &Ty)>)>,
        scopes: Query<&SymbolTable>,
        mut commands: Commands,
    ) {
        let Ok((ref_to_symbol, (as_ref, as_ty))) = refs.get(target.entity) else {
            return;
        };
        let name = as_ref
            .map(|it| &it.ident)
            .or(as_ty.map(|it| &it.ident))
            .cloned()
            .unwrap();
        let description = SymbolDescription::new(Name::new(name), ref_to_symbol.kind);
        let Ok(table) = scopes.get(ref_to_symbol.scope_id) else {
            return;
        };
        let Some(data) = table.get(&description) else {
            return;
        };
        commands
            .entity(target.entity)
            .insert(Declaration(data.bound_node.entity()));
    }

    /// Symbol rules:
    /// - there is should be only one symbol of some kind per scope
    /// - `Identifier::Type` should have `Type` kind
    fn system(
        query: Query<(AnyNodeRef, &Scoped), (SymbolUnionFilter, Added<Scoped>)>,
        mut symbol_tables: Query<(&mut SymbolTable, Option<&Name>, &Scope), Without<Node>>,
        spans: Query<&SourceSpan>,
        reporter: Reporter,
    ) {
        for (node, scoped) in query.iter() {
            let Some(node) = node.to_enum::<SymbolUnion>() else {
                continue;
            };
            let (kind, name) = match node.inner {
                SymbolUnionItem::Const(x) => match &*x {
                    Const::Enum => (Type, x.name().clone()),
                    Const::Struct => (Type, x.name().clone()),
                    Const::Fn => (Function, x.name().clone()),
                    Const::Value => (SymbolKind::Const, x.name().clone()),
                },
                SymbolUnionItem::VarDecl(x) => (Variable, x.name().clone()),
                SymbolUnionItem::EnumConst(x) => (SymbolKind::Const, x.name().clone()),
                SymbolUnionItem::TyParam(x) => (Parameter, x.name().clone()),
                SymbolUnionItem::NonTyParam(x) => (Parameter, x.name().clone()),
            };
            let description = SymbolDescription::new(name.clone(), kind);

            let (mut table, scope_name, scope) = symbol_tables.get_mut(scoped.get()).unwrap();
            match table.entry(description) {
                Entry::Occupied(x) => reporter.report(DuplicatedSymbolError {
                    bound_name: name,
                    scope_name: scope_name.cloned(),
                    scope_start: spans.get(scope.start_from.entity()).unwrap().0,
                    current_def: spans.get(node.id.entity()).unwrap().0,
                    previous_def: spans.get(x.get().bound_node.entity()).unwrap().0,
                }),
                Entry::Vacant(x) => {
                    x.insert(SymbolData::new(node.id));
                }
            };
        }
    }
}
