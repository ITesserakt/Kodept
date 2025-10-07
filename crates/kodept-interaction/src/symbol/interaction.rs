use crate::report::Reporter;
use crate::scope::Scoped;
use crate::scope::storage::Scope;
use crate::symbol::SymbolKind::{self, Function, Parameter, Type, Variable};
use crate::symbol::table::SymbolTable;
use crate::symbol::{Declaration, RefToSymbol, SymbolData, SymbolDescription};
use crate::utils::{Ctx, Disposable, Interaction, wrap_system};
use bevy_ecs::prelude::{
    Added, AnyOf, Commands, Entity, EntityEvent, Insert, IntoSystem, On, Populated, Query,
    SystemSet,
};
use bevy_ecs::relationship::Relationship;
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Name, RequireProperty, SourceSpan};
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

#[derive(Debug, EntityEvent)]
struct CreateSymbolEvent {
    entity: Entity,
    name: Name,
    kind: SymbolKind,
}

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
        ctx.register(wrap_system(
            Self::name(),
            ExtractSymbolsPass::create_symbol_for::<VarDecl>(|id, name, _| CreateSymbolEvent {
                entity: id,
                name: name.clone(),
                kind: Variable,
            }),
        ));
        ctx.register(wrap_system(
            Self::name(),
            ExtractSymbolsPass::create_symbol_for::<EnumConst>(|id, name, _| CreateSymbolEvent {
                entity: id,
                name: name.clone(),
                kind: SymbolKind::Const,
            }),
        ));
        ctx.register(wrap_system(
            Self::name(),
            ExtractSymbolsPass::create_symbol_for::<TyParam>(|id, name, _| CreateSymbolEvent {
                entity: id,
                name: name.clone(),
                kind: Parameter,
            }),
        ));
        ctx.register(wrap_system(
            Self::name(),
            ExtractSymbolsPass::create_symbol_for::<NonTyParam>(|id, name, _| CreateSymbolEvent {
                entity: id,
                name: name.clone(),
                kind: Parameter,
            }),
        ));
        ctx.register(wrap_system(
            Self::name(),
            ExtractSymbolsPass::create_symbol_for::<Const>(|id, name, it| CreateSymbolEvent {
                entity: id,
                name: name.clone(),
                kind: match it {
                    Const::Value => SymbolKind::Const,
                    Const::Enum => Type,
                    Const::Struct => Type,
                    Const::Fn => Function,
                },
            }),
        ));

        (
            ctx.register_observer(Self::sustain_declaration_links),
            ctx.register_observer(Self::on_create_symbol),
        )
    }
}

impl ExtractSymbolsPass {
    fn create_symbol_for<T: ASTNode + RequireProperty<Name>>(
        mut extract: impl FnMut(Entity, &Name, &T) -> CreateSymbolEvent + Send + Sync + 'static,
    ) -> impl IntoSystem<(), (), ()> {
        IntoSystem::into_system(
            move |query: Populated<(Entity, &Name, &T), Added<Scoped>>, mut commands: Commands| {
                for (id, name, item) in query {
                    commands.entity(id).trigger(|id| extract(id, name, item));
                }
            },
        )
    }

    fn on_create_symbol(
        create_symbol: On<CreateSymbolEvent>,
        query: Query<&Scoped>,
        spans: Query<&SourceSpan>,
        mut symbol_tables: Query<(&mut SymbolTable, Option<&Name>, &Scope)>,
        reporter: Reporter,
    ) {
        let description = SymbolDescription::new(create_symbol.name.clone(), create_symbol.kind);
        let scope = query
            .get(create_symbol.entity)
            .expect("Could not find corresponding scope for entity");
        let (mut table, scope_name, scope) = symbol_tables
            .get_mut(scope.get())
            .expect("Could not find symbol table for scope");
        match table.entry(description) {
            Entry::Occupied(x) if x.get().bound_node == create_symbol.entity => {
                // Symbol already exists, so no additional work required
            }
            Entry::Occupied(x) => reporter.report(DuplicatedSymbolError {
                bound_name: create_symbol.name.clone(),
                scope_name: scope_name.cloned(),
                scope_start: spans.get(scope.start_from.entity()).unwrap().0,
                current_def: spans.get(create_symbol.entity).unwrap().0,
                previous_def: spans.get(x.get().bound_node).unwrap().0,
            }),
            Entry::Vacant(x) => {
                x.insert(SymbolData::new(create_symbol.entity));
            }
        }
    }

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
            .insert(Declaration(data.bound_node));
    }
}
