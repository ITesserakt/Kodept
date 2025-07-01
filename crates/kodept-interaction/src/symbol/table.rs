use crate::symbol::{Symbol, SymbolDescription, SymbolKind};
use bevy_ecs::prelude::Component;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// Contains symbols from a specific scope
#[derive(Debug, Component, Default)]
pub(crate) struct SymbolTable(HashMap<Symbol, ()>);

#[derive(Debug)]
pub(crate) enum SymbolSearchError<'s> {
    /// None symbols were found by given name and kind
    NotFound,
    /// Multiple symbols were found for the given name
    Ambiguous(Vec<(&'s Symbol, SymbolKind)>),
}

impl SymbolSearchError<'_> {
    pub(crate) fn as_ambiguous(&self) -> &[(&Symbol, SymbolKind)] {
        match self {
            SymbolSearchError::NotFound => &[],
            SymbolSearchError::Ambiguous(vec) => vec.as_slice(),
        }
    }
}

impl SymbolTable {
    pub(super) fn entry(&mut self, symbol: Symbol) -> Entry<'_, Symbol, ()> {
        self.0.entry(symbol)
    }

    pub(crate) fn get(&self, description: &SymbolDescription) -> Option<&Symbol> {
        self.0.get_key_value(description).map(|it| it.0)
    }
}
