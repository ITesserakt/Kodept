use derive_more::From;
use std::collections::HashMap;
use std::rc::Rc;

type Path = String;

#[derive(From)]
pub enum Symbol {
    Type(TypeSymbol),
    Var(VarSymbol),
}

pub enum TypeSymbol {
    Primitive { name: Path },
    UserDefined { name: Path },
}

pub struct VarSymbol {
    path: Path,
    assigned_type: Option<Rc<TypeSymbol>>,
}

pub struct ScopedSymbolTable {
    symbols: HashMap<Path, Symbol>,
    enclosing_scope: Option<Box<ScopedSymbolTable>>,
    name: String,
    depth: usize,
}

impl Symbol {
    pub fn path(&self) -> &Path {
        match self {
            Symbol::Type(x) => match x {
                TypeSymbol::Primitive { name, .. } => name,
                TypeSymbol::UserDefined { name, .. } => name,
            },
            Symbol::Var(x) => &x.path,
        }
    }
}

impl ScopedSymbolTable {
    pub fn lookup(&self, path: &Path, exclusive: bool) -> Option<&Symbol> {
        let mut current_scope = Some(self);
        loop {
            if let Some(scope) = current_scope {
                match scope.symbols.get(path) {
                    None if exclusive => return None,
                    None => current_scope = scope.enclosing_scope.as_deref(),
                    x @ Some(_) => return x,
                }
            } else {
                return None;
            }
        }
    }

    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.path().clone(), symbol);
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
}
