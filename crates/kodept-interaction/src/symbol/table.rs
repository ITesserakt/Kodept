use crate::symbol::SymbolKind::{Function, Parameter, Variable};
use crate::symbol::{Symbol, SymbolDescription, SymbolKind};
use bevy_ecs::prelude::{Component, Name};
use hashbrown::hash_set::Entry;
use hashbrown::{DefaultHashBuilder, HashSet};
use SymbolKind::Type;

/// Contains symbols from a specific scope
#[derive(Debug, Component, Default)]
pub(crate) struct SymbolTable(HashSet<Symbol>);

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
    pub(super) fn entry(&mut self, symbol: Symbol) -> Entry<'_, Symbol, DefaultHashBuilder> {
        self.0.entry(symbol)
    }
    
    pub(crate) fn get_type(&self, name: Name) -> Option<&Symbol> {
        self.get(name, Type)
    }

    pub(crate) fn get_value(&self, name: Name) -> Result<&'_ Symbol, SymbolSearchError<'_>> {
        let mut description = SymbolDescription::new(name, Variable);
        let symbol_as_var = self.0.get(&description);
        description.kind = Function;
        let symbol_as_func = self.0.get(&description);
        description.kind = Parameter;
        let symbol_as_param = self.0.get(&description);
        match (symbol_as_var, symbol_as_func, symbol_as_param) {
            (Some(x), None, None) | (None, Some(x), None) | (None, None, Some(x)) => Ok(x),
            (Some(a), Some(b), Some(c)) => Err(SymbolSearchError::Ambiguous(vec![
                (a, Variable),
                (b, Function),
                (c, Parameter),
            ])),
            (Some(a), Some(b), None) => Err(SymbolSearchError::Ambiguous(vec![
                (a, Variable),
                (b, Function),
            ])),
            (Some(a), None, Some(b)) => Err(SymbolSearchError::Ambiguous(vec![
                (a, Variable),
                (b, Parameter),
            ])),
            (None, Some(a), Some(b)) => Err(SymbolSearchError::Ambiguous(vec![
                (a, Function),
                (b, Parameter),
            ])),
            (None, None, None) => Err(SymbolSearchError::NotFound),
        }
    }

    pub(crate) fn get(&self, name: Name, kind: SymbolKind) -> Option<&Symbol> {
        let description = SymbolDescription::new(name, kind);
        self.0.get(&description)
    }
}
