pub(crate) mod interaction;
pub(crate) mod table;

use crate::scope::Visibility;
use crate::symbol::table::SymbolTable;
use bevy_ecs::prelude::{Component, Entity, Name};
use kodept_ast::prelude::NodeId;
use kodept_ast_nodes::term::Ref;
use kodept_inference::r#type::PolymorphicType;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

#[derive(Debug, Component)]
pub(crate) struct RefToSymbol {
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub scope_id: Entity,
}

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

#[derive(Debug, Eq)]
pub(crate) struct Symbol {
    pub description: SymbolDescription,
    pub bound_node: NodeId,
    pub ty: OnceLock<PolymorphicType>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SymbolDescription {
    name: Name,
    pub kind: SymbolKind,
    visibility: Visibility,
}

impl RefToSymbol {
    pub(crate) fn new(scope_id: Entity, description: &SymbolDescription) -> Self {
        Self {
            kind: description.kind,
            scope_id,
            visibility: description.visibility,
        }
    }

    pub(crate) fn resolve<'a>(&self, reference: &Ref, table: &'a SymbolTable) -> &'a Symbol {
        let description =
            SymbolDescription::new(Name::new(reference.ident.name().to_string()), self.kind);
        table.get(&description).unwrap()
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
    pub(crate) fn new(kind: SymbolKind, bound_node: NodeId, name: Name) -> Self {
        Self {
            description: SymbolDescription {
                name,
                kind,
                visibility: Default::default(),
            },
            bound_node,
            ty: OnceLock::new(),
        }
    }

    pub(crate) fn with_type(&mut self, ty: PolymorphicType) {
        _ = self.ty.set(ty);
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
