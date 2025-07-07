use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::Scoped;
use crate::symbol::table::SymbolTable;
use crate::symbol::SymbolKind::{self, Function, Parameter, Type, Variable};
use crate::symbol::{SymbolData, SymbolDescription};
use crate::wrapper::InteractionExt;
use crate::{done, Ctx, Interaction};
use bevy_ecs::prelude::{
    any_match_filter, Added, Commands, Query, ResMut, Resource, SystemSet, Without,
};
use bevy_ecs::relationship::Relationship;
use bevy_ecs::schedule::IntoScheduleConfigs;
use kodept_ast::define_union;
use kodept_ast::prelude::AnyNodeRef;
use kodept_ast::properties::{Name, Node, SourceSpan};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::consts::Const;
use kodept_ast_nodes::top_level::EnumConst;
use kodept_ast_nodes::types::{NonTyParam, TyParam};
use kodept_core::code_point::Span;
use kodept_report::message::{Diagnostic, Label, Severity};
use kodept_report::traits::IntoSpannedReportMessage;
use std::borrow::Cow;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Eq, PartialEq, Hash, SystemSet)]
pub struct ExtractSymbolsPass;

#[derive(Debug, Resource, Default, PartialEq)]
pub(crate) struct ExtractSymbolsLock(bool);

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

impl ExtractSymbolsLock {
    pub(crate) const UNLOCKED: Self = ExtractSymbolsLock(false);
}

impl IntoSpannedReportMessage for DuplicatedSymbolError {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Error)
            .with_message(format!(
                "Element with name `{}` already defined",
                self.bound_name
            ))
            .with_label(Label::primary("", self.current_def))
            .with_label(Label::secondary("previous declaration", self.previous_def))
            .with_label(Label::secondary(
                if let Some(name) = self.scope_name {
                    Cow::Owned(format!("in scope `{name}`"))
                } else {
                    "in scope".into()
                },
                self.scope_start,
            ))
    }
}

impl Interaction for ExtractSymbolsPass {
    type Error = DuplicatedSymbolError;

    fn install(ctx: &mut Ctx) {
        ctx.register(Self::disable_lock_system);

        ctx.register(
            Self::wrap_system(Self::system)
                .run_if(any_match_filter::<(SymbolUnionFilter, Added<Scoped>)>)
                .after(Self::disable_lock_system)
                .in_set(ExtractSymbolsPass),
        );
    }
}

impl ExtractSymbolsPass {
    fn disable_lock_system(lock: Option<ResMut<ExtractSymbolsLock>>) {
        if let Some(mut lock) = lock {
            lock.0 = false;
        }
    }

    /// Symbol rules:
    /// - there is should be only one symbol of some kind per scope
    /// - `Identifier::Type` should have `Type` kind
    fn system(
        query: Query<(AnyNodeRef, &Scoped), (SymbolUnionFilter, Added<Scoped>)>,
        mut symbol_tables: Query<(&mut SymbolTable, Option<&Name>, &Scope), Without<Node>>,
        spans: Query<&SourceSpan>,
        reporter: Reporter,
        mut commands: Commands,
    ) -> crate::Result<DuplicatedSymbolError> {
        commands.insert_resource(ExtractSymbolsLock(true));

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

        done()
    }
}
