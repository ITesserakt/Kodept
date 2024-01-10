use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::symbol::{Symbol, TypeSymbol, VarSymbol};
use crate::Errors::{AlreadyDefined, UnresolvedReference};
use crate::{Errors, Path};
use replace_with::replace_with_or_abort;

pub struct ScopedSymbolTable {
    symbols: HashMap<Path, Symbol>,
    enclosing_scope: Option<Box<ScopedSymbolTable>>,
    name: String,
    depth: usize,
}

impl ScopedSymbolTable {
    pub fn lookup(&self, path: &Path, exclusive: bool) -> Result<Symbol, Errors> {
        let mut current_scope = Some(self);
        loop {
            if let Some(scope) = current_scope {
                match scope.symbols.get(path) {
                    None if exclusive => return Err(UnresolvedReference(path.clone())),
                    None => current_scope = scope.enclosing_scope.as_deref(),
                    Some(x) => return Ok(x.clone()),
                }
            } else {
                return Err(UnresolvedReference(path.clone()));
            }
        }
    }

    pub fn lookup_type(&self, path: &Path, exclusive: bool) -> Result<Rc<TypeSymbol>, Errors> {
        self.lookup(path, exclusive)
            .and_then(|it| it.try_into().map_err(|_| UnresolvedReference(path.clone())))
    }

    pub fn lookup_var(&self, path: &Path, exclusive: bool) -> Result<Rc<VarSymbol>, Errors> {
        self.lookup(path, exclusive)
            .and_then(|it| it.try_into().map_err(|_| UnresolvedReference(path.clone())))
    }

    pub fn insert<T: Clone + Into<Symbol>>(&mut self, symbol: T) -> Result<T, Errors> {
        let out = symbol;
        let symbol = out.clone().into();
        match self.symbols.insert(symbol.path().clone(), symbol.clone()) {
            None => Ok(out),
            Some(x) => Err(AlreadyDefined(x.path().clone())),
        }
    }

    pub fn new(name: impl Into<String>, enclosing: Option<ScopedSymbolTable>) -> Self {
        let depth = enclosing.as_ref().map(|s| s.depth + 1).unwrap_or(0);
        Self {
            symbols: Default::default(),
            enclosing_scope: enclosing.map(Box::new),
            name: name.into(),
            depth,
        }
    }

    pub fn new_layer(&mut self, name: Path) {
        replace_with_or_abort(self, |this| ScopedSymbolTable::new(name, Some(this)));
    }

    pub fn replace_with_enclosing_scope(&mut self, name: &Path) -> Result<(), Errors> {
        if &self.name != name {
            return Err(Errors::WrongScope {
                expected: self.name.clone(),
                found: name.clone(),
            });
        }
        if self.enclosing_scope.is_some() {
            replace_with_or_abort(self, |this| {
                *this
                    .enclosing_scope
                    .expect("Enclosing scope should be some")
            })
        }
        Ok(())
    }
}

impl Debug for ScopedSymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopedSymbolTable")
            .field("name", &self.name)
            .field("depth", &self.depth)
            .field("enclosing_scope", &self.enclosing_scope)
            .field("symbols", &self.symbols)
            .finish()
    }
}
