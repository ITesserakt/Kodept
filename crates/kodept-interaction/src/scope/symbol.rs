use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::symbol::SymbolValueKind::{Function, Parameter, Variable};
use crate::scope::{ScopeMapping, Visibility};
use crate::{done, Interaction, InteractionWrapper, Result};
use bevy_ecs::entity::EntityHashMap;
use bevy_ecs::prelude::{Commands, Component, DetectChanges, Query, Res};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_hierarchy::Children;
use hashbrown::hash_set::Entry;
use hashbrown::HashSet;
use kodept_ast::prelude::{AnyNodeRef, Arity, IntoEnum, NodeId};
use kodept_ast::properties::Name;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::children::arity::Singular;
use kodept_ast::{define_union, Str};
use kodept_ast_nodes::block_level::VarDecl;
use kodept_ast_nodes::constants::Const;
use kodept_ast_nodes::function::Func;
use kodept_ast_nodes::top_level::{EnumConst, EnumDecl, StructDecl};
use kodept_ast_nodes::types::{NonTyParam, TyParam};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_inference::r#type::PolymorphicType;
use kodept_report::error::report::{IntoSpannedReportMessage, Label, Severity};
use kodept_report::error::Diagnostic;
use std::borrow::{Borrow, Cow};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;
use SymbolKind::Value;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum SymbolKind {
    Type,
    Value(SymbolValueKind),
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum SymbolValueKind {
    Variable,
    Parameter,
    Function,
}

#[derive(Debug, Component, Eq)]
pub struct Symbol {
    pub description: SymbolDescription,
    pub bound_node: NodeId,
    pub ty: OnceLock<PolymorphicType>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SymbolDescription {
    name: Str,
    kind: SymbolKind,
    visibility: Visibility,
}

#[derive(Debug, Component)]
pub(super) struct SymbolTable(HashSet<Symbol>);

impl SymbolTable {
    pub(super) fn get_type(&self, name: &impl ToOwned<Owned = Str>) -> Option<&Symbol> {
        self.0
            .get(&SymbolDescription::new(name.to_owned(), SymbolKind::Type))
    }

    pub(super) fn get_value(&self, name: &impl ToOwned<Owned = Str>) -> Option<&Symbol> {
        self.0
            .get(&SymbolDescription::new(name.to_owned(), Value(Function)))
            .or_else(|| {
                self.0
                    .get(&SymbolDescription::new(name.to_owned(), Value(Variable)))
            })
            .or_else(|| {
                self.0
                    .get(&SymbolDescription::new(name.to_owned(), Value(Parameter)))
            })
    }
}

impl SymbolDescription {
    pub fn new(name: Str, kind: SymbolKind) -> Self {
        Self {
            name,
            kind,
            visibility: Default::default(),
        }
    }
}

impl Deref for SymbolTable {
    type Target = HashSet<Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SymbolTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.description.hash(state);
    }
}

impl Borrow<SymbolDescription> for Symbol {
    fn borrow(&self) -> &SymbolDescription {
        &self.description
    }
}

impl Symbol {
    pub fn new(kind: SymbolKind, bound_node: NodeId, name: impl Into<Str>) -> Self {
        Self {
            description: SymbolDescription {
                name: name.into(),
                kind,
                visibility: Default::default(),
            },
            bound_node,
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
    fn extract_symbol(
        node: SymbolUnion,
        children: &Children,
        query: &Query<(AnyNodeRef, &Children), SymbolUnionFilter>,
    ) -> Symbol {
        define_union!(enum ConstUnion[ConstUnionItem] {
            EnumDecl | StructDecl | Func
        });

        let id = node.id;
        match &*node {
            SymbolUnionItem::StructDecl(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::EnumDecl(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::Func(x) => Symbol::new(Value(Function), id, x.name().clone()),
            SymbolUnionItem::VarDecl(x) => Symbol::new(Value(Variable), id, x.name().clone()),
            SymbolUnionItem::EnumConst(x) => Symbol::new(SymbolKind::Type, id, x.name().clone()),
            SymbolUnionItem::TyParam(x) => Symbol::new(Value(Parameter), id, x.name().clone()),
            SymbolUnionItem::NonTyParam(x) => Symbol::new(Value(Parameter), id, x.name().clone()),
            SymbolUnionItem::Const(_) => {
                let iter = children
                    .iter()
                    .filter_map(|id| query.get(*id).ok().and_then(|(it, _)| it.into_enum()));
                let node: ConstUnion = Singular::try_from_iter(iter).unwrap();
                match &*node {
                    ConstUnionItem::EnumDecl(x) => {
                        Symbol::new(SymbolKind::Type, id, x.name().clone())
                    }
                    ConstUnionItem::StructDecl(x) => {
                        Symbol::new(SymbolKind::Type, id, x.name().clone())
                    }
                    ConstUnionItem::Func(x) => Symbol::new(Value(Function), id, x.name().clone()),
                }
            }
        }
    }

    fn system(
        query: Query<(AnyNodeRef, &Children), SymbolUnionFilter>,
        scope_mapping: Res<ScopeMapping>,
        mut commands: Commands,
        reporter: Reporter,
        syntax: Res<SyntaxResolver>,
        scopes: Query<(&Scope, Option<&Name>)>,
    ) -> Result<DuplicatedSymbolError> {
        let mut symbols: EntityHashMap<HashSet<Symbol>> = EntityHashMap::default();

        let iter = query.iter().filter_map(|(node, children)| {
            let node = node.into_enum()?;
            Some(Self::extract_symbol(node, children, &query))
        });
        for symbol in iter {
            let bound_node = symbol.bound_node;
            let enclosing_scope = scope_mapping.enclosing_scope_id(bound_node);
            let set = symbols.entry(enclosing_scope).or_default();
            match set.entry(symbol) {
                Entry::Occupied(x) => {
                    let current_def_location =
                        syntax.try_get_unknown(bound_node).unwrap().location();
                    let previous_def_location = syntax
                        .try_get_unknown(x.get().bound_node)
                        .unwrap()
                        .location();
                    let scope = scopes.get(enclosing_scope).unwrap();
                    let scope_start_location = syntax
                        .try_get_unknown(scope.0.start_from)
                        .unwrap()
                        .location();
                    let scope_name = scope.1.map(|it| &it.name).cloned();
                    let error = DuplicatedSymbolError {
                        bound_name: x.get().description.name.clone(),
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
