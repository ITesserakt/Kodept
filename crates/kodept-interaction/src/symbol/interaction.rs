use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::Scoped;
use crate::symbol::table::SymbolTable;
use crate::symbol::SymbolKind::{Function, Parameter, Variable};
use crate::symbol::{Symbol, SymbolKind};
use crate::wrapper::InteractionWrapper;
use crate::{done, Interaction};
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::hash_map::EntityHashMap;
use bevy_ecs::prelude::{Children, Commands, IntoScheduleConfigs, Populated, Query};
use bevy_ecs::query::Changed;
use bevy_ecs::relationship::Relationship;
use hashbrown::hash_set::Entry;
use hashbrown::HashSet;
use kodept_ast::properties::Name;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::prelude::ASTQuery;
use kodept_ast::{define_union, Str};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::constants::Const;
use kodept_ast_nodes::function::Func;
use kodept_ast_nodes::top_level::{EnumConst, EnumDecl, StructDecl};
use kodept_ast_nodes::types::{NonTyParam, TyParam};
use kodept_core::code_point::CodePoint;
use kodept_report::message::{Diagnostic, Label, Severity};
use kodept_report::traits::IntoSpannedReportMessage;
use std::borrow::Cow;

pub struct ExtractSymbols;

define_union!(enum SymbolUnion[SymbolUnionItem, SymbolUnionFilter] {
    StructDecl | EnumDecl | Func | Const | VarDecl | EnumConst | TyParam| NonTyParam
});

#[derive(Debug)]
pub struct DuplicatedSymbolError {
    bound_name: Str,
    scope_name: Option<Str>,
    scope_start_location: CodePoint,
    current_def_location: CodePoint,
    previous_def_location: CodePoint,
}

impl IntoSpannedReportMessage for DuplicatedSymbolError {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Error)
            .with_message(format!(
                "Element with name `{}` already defined",
                self.bound_name
            ))
            .with_label(Label::primary("", self.current_def_location))
            .with_label(Label::secondary(
                "previous declaration",
                self.previous_def_location,
            ))
            .with_label(Label::secondary(
                if let Some(name) = self.scope_name {
                    Cow::Owned(format!("in scope `{name}`"))
                } else {
                    "in scope".into()
                },
                self.scope_start_location,
            ))
    }
}

impl Interaction for ExtractSymbols {
    type Error = DuplicatedSymbolError;

    fn interaction() -> InteractionWrapper<Self::Error> {
        let config = InteractionWrapper::wrap(Self::system)
            .unwrap()
            .run_if(|query: Populated<(), Changed<Scoped>>| true);
        InteractionWrapper::from_configs(config)
    }
}

impl ExtractSymbols {
    fn extract_symbol(node: SymbolUnion, query: &ASTQuery<SymbolUnionFilter>) -> Symbol {
        define_union!(enum ConstUnion[ConstUnionItem] {
            EnumDecl | StructDecl | Func
        });

        let id = node.id;
        match &*node {
            SymbolUnionItem::StructDecl(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::EnumDecl(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::Func(x) => Symbol::new(Function, id, x.name().clone()),
            SymbolUnionItem::VarDecl(x) => Symbol::new(Variable, id, x.name().clone()),
            SymbolUnionItem::EnumConst(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::TyParam(x) => Symbol::new(Parameter, id, x.name().clone()),
            SymbolUnionItem::NonTyParam(x) => Symbol::new(Parameter, id, x.name().clone()),
            SymbolUnionItem::Const(x) => {
                if let Ok(Some(x)) = query.children_as::<_, EnumDecl, _>(x.id()) {
                    Symbol::new(SymbolKind::Type, id, x.name().clone())
                } else if let Ok(Some(x)) = query.children_as::<_, StructDecl, _>(x.id()) {
                    Symbol::new(SymbolKind::Type, id, x.name().clone())
                } else if let Ok(Some(x)) = query.children_as::<_, Func, _>(x.id()) {
                    Symbol::new(Function, id, x.name().clone())
                } else {
                    unreachable!()
                }
            }
        }
    }

    fn system(
        query: ASTQuery<SymbolUnionFilter>,
        enclosing_scopes: Query<&Scoped>,
        mut commands: Commands,
        reporter: Reporter,
        syntax: Res<SyntaxResolver>,
        scopes: Query<(&Scope, Option<&Name>, Option<&Children>)>,
    ) -> crate::Result<DuplicatedSymbolError> {
        let mut symbols: EntityHashMap<HashSet<Symbol>> = EntityHashMap::default();

        let iter = query.iter_enum().map(|it| Self::extract_symbol(it, &query));
        for symbol in iter {
            let bound_node = symbol.bound_node;
            let enclosing_scope = enclosing_scopes.get(bound_node.entity()).unwrap().get();
            let set = symbols.entry(enclosing_scope).or_default();
            match set.entry(symbol) {
                Entry::Occupied(x) => {
                    let current_def_location = syntax.get_location(bound_node);
                    let previous_def_location = syntax.get_location(x.get().bound_node);
                    let (scope, name, _) = scopes.get(enclosing_scope).unwrap();
                    let scope_start_location = syntax.get_location(scope.start_from);
                    let error = DuplicatedSymbolError {
                        bound_name: x.get().description.name.clone(),
                        scope_start_location,
                        scope_name: name.map(|it| &it.0).cloned(),
                        current_def_location,
                        previous_def_location,
                    };
                    reporter.report(error);
                    continue;
                }
                Entry::Vacant(x) => x.insert(),
            }
        }

        for (scope_id, set) in symbols {
            commands.entity(scope_id).insert(SymbolTable::new(set));
        }

        done()
    }
}
