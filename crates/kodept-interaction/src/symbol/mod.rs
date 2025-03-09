pub(crate) mod table;
pub(crate) mod interaction;

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use kodept_ast::external::Component;
use kodept_ast::prelude::NodeId;
use kodept_ast::Str;
use kodept_inference::r#type::PolymorphicType;
use crate::scope::Visibility;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub(crate) enum SymbolKind {
    Type,
    Variable,
    Parameter,
    Function,
}

#[derive(Debug, Component, Eq)]
pub(crate) struct Symbol {
    pub description: SymbolDescription,
    pub bound_node: NodeId,
    pub ty: OnceLock<PolymorphicType>,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct SymbolDescription {
    name: Str,
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
    pub(crate) fn new(kind: SymbolKind, bound_node: NodeId, name: impl Into<Str>) -> Self {
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

    pub(crate) fn with_type(&mut self, ty: PolymorphicType) {
        _ = self.ty.set(ty);
    }
}

impl SymbolDescription {
    pub(crate) fn new(name: Str, kind: SymbolKind) -> Self {
        Self {
            name,
            kind,
            visibility: Default::default(),
        }
    }
}
