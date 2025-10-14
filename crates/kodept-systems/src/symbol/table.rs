use crate::symbol::{SymbolData, SymbolDescription};
use bevy_ecs::prelude::Component;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// Contains symbols from a specific scope
#[derive(Debug, Component, Default)]
pub(crate) struct SymbolTable(HashMap<SymbolDescription, SymbolData>);

impl SymbolTable {
    pub(super) fn entry(
        &mut self,
        symbol: SymbolDescription,
    ) -> Entry<'_, SymbolDescription, SymbolData> {
        self.0.entry(symbol)
    }

    pub(crate) fn get(&self, description: &SymbolDescription) -> Option<&SymbolData> {
        self.0.get(description)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&SymbolDescription, &SymbolData)> {
        self.0.iter()
    }

    pub(crate) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&SymbolDescription, &mut SymbolData)> {
        self.0.iter_mut()
    }
}
