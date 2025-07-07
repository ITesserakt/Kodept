use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::Scoped;
use crate::symbol::table::SymbolTable;
use crate::symbol::SymbolKind::{Const, Function, Parameter, Type, Variable};
use crate::symbol::{SymbolData, SymbolDescription};
use crate::wrapper::InteractionExt;
use crate::{done, Ctx, Interaction};
use bevy_ecs::prelude::{any_match_filter, Added, Query, SystemSet, Without};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::schedule::IntoScheduleConfigs;
use kodept_ast::define_union;
use kodept_ast::prelude::AnyNodeRef;
use kodept_ast::properties::{Name, Node, SourceSpan};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::function::FuncSignature;
use kodept_ast_nodes::top_level::{EnumConst, EnumDecl, StructDecl};
use kodept_ast_nodes::types::{NonTyParam, TyParam};
use kodept_core::code_point::Span;
use kodept_diagnostic_macros::Diagnostic;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemSet)]
pub struct ExtractSymbolsPass;

define_union!(enum SymbolUnion[SymbolUnionItem, SymbolUnionFilter] {
    StructDecl | EnumDecl | FuncSignature | VarDecl | EnumConst | TyParam | NonTyParam
});

#[derive(Diagnostic)]
#[severity("error")]
#[message("Element with name `{bound_name}` already defined")]
pub struct DuplicatedSymbolError {
    #[primary_label]
    current_def: Span,
    #[secondary_label("previous declaration")]
    previous_def: Span,
    bound_name: Name,
    #[secondary_label("in scope {}", self.scope_name.as_deref().unwrap_or(""))]
    scope_start: Span,
    scope_name: Option<Name>,
}

impl Interaction for ExtractSymbolsPass {
    type Error = DuplicatedSymbolError;

    fn install(ctx: &mut Ctx) {
        ctx.register(
            Self::wrap_system(Self::system)
                .run_if(any_match_filter::<(SymbolUnionFilter, Added<Scoped>)>)
                .in_set(ExtractSymbolsPass),
        );
    }
}

impl ExtractSymbolsPass {
    /// Symbol rules:
    /// - there is should be only one symbol of some kind per scope
    /// - `Identifier::Type` should have `Type` kind
    fn system(
        query: Query<(AnyNodeRef, &Scoped), (SymbolUnionFilter, Added<Scoped>)>,
        mut symbol_tables: Query<(&mut SymbolTable, Option<&Name>, &Scope), Without<Node>>,
        spans: Query<&SourceSpan>,
        reporter: Reporter,
    ) -> crate::Result<DuplicatedSymbolError> {
        for (node, scoped) in query.iter() {
            let Some(node) = node.to_enum::<SymbolUnion>() else {
                continue;
            };
            let (kind, name) = match node.inner {
                SymbolUnionItem::StructDecl(x) => (Type, x.name().clone()),
                SymbolUnionItem::EnumDecl(x) => (Type, x.name().clone()),
                SymbolUnionItem::FuncSignature(x) => (Function, x.name().clone()),
                SymbolUnionItem::VarDecl(x) => (Variable, x.name().clone()),
                SymbolUnionItem::EnumConst(x) => (Const, x.name().clone()),
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

        done()
    }
}
