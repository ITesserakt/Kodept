use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::{ScopeMapping, Visibility};
use crate::{done, Interaction, InteractionWrapper, Result};
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Commands, Component, DetectChanges, Query, Res};
use bevy_ecs::schedule::IntoSystemConfigs;
use hashbrown::hash_set::Entry;
use hashbrown::HashSet;
use kodept_ast::prelude::{AnyNodeRef, IntoEnum, NodeId};
use kodept_ast::properties::Name;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::{define_union, Str};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::function::Func;
use kodept_ast_nodes::top_level::{EnumConst, EnumDecl, StructDecl};
use kodept_ast_nodes::types::{NonTyParam, TyParam};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_inference::r#type::PolymorphicType;
use kodept_report::error::report::{IntoSpannedReportMessage, Label, Severity};
use kodept_report::error::Diagnostic;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use kodept_ast_nodes::constants::Const;

#[derive(Debug, PartialEq, Hash, Eq)]
pub enum SymbolKind {
    Type,
    Variable,
    Parameter,
    Function,
}

#[derive(Debug, Component, Eq)]
pub struct Symbol {
    pub visibility: Visibility,
    pub kind: SymbolKind,
    pub bound_node: NodeId,
    pub name: Str,
    pub ty: OnceLock<PolymorphicType>,
}

#[derive(Debug, Component)]
pub struct SymbolTable(HashSet<Symbol>);

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.name.hash(state);
    }
}

impl Symbol {
    pub fn new(kind: SymbolKind, bound_node: NodeId, name: impl Into<Str>) -> Self {
        Self {
            visibility: Visibility::Private,
            kind,
            bound_node,
            name: name.into(),
            ty: OnceLock::new(),
        }
    }

    pub fn with_type(&mut self, ty: PolymorphicType) {
        _ = self.ty.set(ty);
    }
}

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
            .run_if(|mapping: Option<Res<ScopeMapping>>| mapping.is_some_and(|it| it.is_changed()));
        InteractionWrapper::from_configs(config)
    }
}

impl ExtractSymbols {
    fn extract_symbol(node: SymbolUnion) -> Symbol {
        let id = node.id;
        match &*node {
            SymbolUnionItem::StructDecl(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::EnumDecl(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::Func(x) => Symbol::new(SymbolKind::Function, id, x.name().clone()),
            SymbolUnionItem::VarDecl(x) => Symbol::new(SymbolKind::Variable, id, x.name().clone()),
            SymbolUnionItem::EnumConst(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::TyParam(x) => Symbol::new(SymbolKind::Parameter, id, x.name().clone()),
            SymbolUnionItem::NonTyParam(x) => {
                Symbol::new(SymbolKind::Parameter, id, x.name().clone())
            }
            SymbolUnionItem::Const(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
        }
    }

    fn system(
        query: Query<AnyNodeRef, SymbolUnionFilter>,
        scope_mapping: Res<ScopeMapping>,
        mut commands: Commands,
        reporter: Reporter,
        syntax: Res<SyntaxResolver>,
        scopes: Query<(&Scope, Option<&Name>)>,
    ) -> Result<DuplicatedSymbolError> {
        let mut symbols: EntityHashMap<HashSet<Symbol>> = EntityHashMap::default();

        let iter = query
            .into_iter()
            .filter_map(|it| it.into_enum())
            .map(Self::extract_symbol);
        for symbol in iter {
            let bound_node = symbol.bound_node;
            let enclosing_scope = scope_mapping.enclosing_scope_id(bound_node);
            let set = symbols.entry(enclosing_scope).or_default();
            match set.entry(symbol) {
                Entry::Occupied(x) => {
                    let current_def_location = syntax.get_unknown(bound_node).unwrap().location();
                    let previous_def_location =
                        syntax.get_unknown(x.get().bound_node).unwrap().location();
                    let scope = scopes.get(enclosing_scope).unwrap();
                    let scope_start_location =
                        syntax.get_unknown(scope.0.start_from).unwrap().location();
                    let scope_name = scope.1.map(|it| &it.name).cloned();
                    let error = DuplicatedSymbolError {
                        bound_name: x.get().name.clone(),
                        scope_start_location,
                        scope_name,
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
            commands.entity(scope_id).insert(SymbolTable(set));
        }

        done()
    }
}
