pub(crate) mod interaction;
pub(crate) mod table;

use crate::scope::Visibility;
use crate::symbol::table::SymbolTable;
use bevy_ecs::prelude::{Component, Entity, Name};
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast_nodes::term::Ref;
use kodept_inference::r#type::PolymorphicType;
use std::hash::Hash;
use bevy_ecs::entity::EntityHashSet;

#[derive(Debug, Component)]
pub(crate) struct RefToSymbol {
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub scope_id: Entity,
}

#[derive(Debug, Component)]
#[relationship(relationship_target = Usages)]
pub struct Declaration(pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = Declaration)]
pub struct Usages(EntityHashSet);

#[derive(Debug, Component, Copy, Clone)]
pub(crate) struct DeferRefResolution;

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub(crate) enum SymbolKind {
    Type,
    Const,
    Variable,
    Parameter,
    Function,
}

#[derive(Debug)]
pub(crate) struct SymbolData {
    pub bound_node: NodeId,
    ty: Option<PolymorphicType>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SymbolDescription {
    name: Name,
    pub kind: SymbolKind,
    visibility: Visibility,
}

impl SymbolData {
    pub(crate) fn new(bound_node: impl Erase) -> Self {
        Self {
            bound_node: bound_node.erase(),
            ty: None,
        }
    }

    pub(crate) fn with_type(mut self, ty: impl Into<PolymorphicType>) -> Self {
        self.ty = Some(ty.into());
        self
    }

    pub(crate) fn set_type(&mut self, ty: PolymorphicType) {
        _ = self.ty.insert(ty);
    }

    pub(crate) fn is_type_known(&self) -> bool {
        self.ty.is_some()
    }

    pub(crate) fn get_type(&self) -> Option<&PolymorphicType> {
        self.ty.as_ref()
    }
}

impl RefToSymbol {
    pub(crate) fn new(scope_id: Entity, description: &SymbolDescription) -> Self {
        Self {
            kind: description.kind,
            scope_id,
            visibility: description.visibility,
        }
    }

    pub(crate) fn resolve<'a>(
        &self,
        reference: &Ref,
        table: &'a SymbolTable,
    ) -> (SymbolDescription, &'a SymbolData) {
        let description = SymbolDescription::new(Name::new(reference.ident.clone()), self.kind);
        let symbol = table.get(&description).unwrap();
        (description, symbol)
    }
}

impl SymbolDescription {
    pub(crate) fn new(name: Name, kind: SymbolKind) -> Self {
        Self {
            name,
            kind,
            visibility: Default::default(),
        }
    }

    pub(crate) fn iter_kinds(&mut self) -> bool {
        match self.kind {
            SymbolKind::Type => {
                self.kind = SymbolKind::Const;
                true
            }
            SymbolKind::Const => false,
            SymbolKind::Variable => {
                self.kind = SymbolKind::Parameter;
                true
            }
            SymbolKind::Parameter => {
                self.kind = SymbolKind::Function;
                true
            }
            SymbolKind::Function => false,
        }
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.kind = match self.kind {
            SymbolKind::Parameter => SymbolKind::Variable,
            SymbolKind::Function => SymbolKind::Variable,
            SymbolKind::Const => SymbolKind::Type,
            _ => self.kind,
        };
    }
}
