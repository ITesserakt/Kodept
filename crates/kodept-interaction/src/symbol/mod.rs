pub(crate) mod interaction;
pub(crate) mod table;

use crate::scope::Visibility;
use bevy_ecs::prelude::Name;
use kodept_ast::prelude::NodeId;
use kodept_inference::r#type::PolymorphicType;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub(crate) enum SymbolKind {
    Type,
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
    kind: SymbolKind,
    visibility: Visibility,
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
}
