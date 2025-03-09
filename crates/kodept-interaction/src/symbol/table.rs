use crate::symbol::SymbolKind::{Function, Parameter, Variable};
use crate::symbol::{Symbol, SymbolDescription, SymbolKind};
use hashbrown::HashSet;
use kodept_ast::external::Component;
use kodept_ast::Str;
use SymbolKind::Type;

#[derive(Debug, Component)]
/// Contains symbols from a specific scope
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
    pub(crate) fn empty() -> Self {
        Self(HashSet::new())
    }

    pub(crate) fn new(set: HashSet<Symbol>) -> Self {
        Self(set)
    }

    pub(crate) fn get_type<T>(&self, name: T) -> Option<&Symbol>
    where
        T: Into<Str>,
    {
        self.get(name, Type)
    }

    pub(crate) fn get_value<T>(&self, name: T) -> Result<&Symbol, SymbolSearchError>
    where
        T: Into<Str>,
    {
        let mut description = SymbolDescription::new(name.into(), Variable);
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

    pub(crate) fn get<T>(&self, name: T, kind: SymbolKind) -> Option<&Symbol>
    where
        T: Into<Str>,
    {
        let description = SymbolDescription::new(name.into(), kind);
        self.0.get(&description)
    }
}
