use crate::Path;
use derive_more::{From, TryInto};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(From, Debug, TryInto, Clone)]
pub enum Symbol {
    Type(Rc<TypeSymbol>),
    Var(Rc<VarSymbol>),
}

#[derive(Debug)]
pub enum TypeSymbol {
    Primitive {
        name: Path,
        kind: PrimitiveTypeSymbol,
    },
    UserDefined {
        name: Path,
    },
}

#[derive(Debug)]
pub enum PrimitiveTypeSymbol {
    Number,
    Char,
}

pub struct VarSymbol {
    path: Path,
    assigned_type: Option<Rc<TypeSymbol>>,
}

impl VarSymbol {
    pub fn new(path: Path, assigned_type: Option<Rc<TypeSymbol>>) -> Rc<Self> {
        Rc::new(Self {
            path,
            assigned_type,
        })
    }
}

impl TypeSymbol {
    pub fn primitive(name: Path, kind: PrimitiveTypeSymbol) -> Rc<TypeSymbol> {
        Rc::new(TypeSymbol::Primitive { name, kind })
    }

    pub fn user(name: Path) -> Rc<TypeSymbol> {
        Rc::new(TypeSymbol::UserDefined { name })
    }
}

impl Symbol {
    pub fn path(&self) -> &Path {
        match self {
            Symbol::Type(x) => match x.as_ref() {
                TypeSymbol::Primitive { name, .. } => name,
                TypeSymbol::UserDefined { name, .. } => name,
            },
            Symbol::Var(x) => &x.path,
        }
    }
}

impl Debug for VarSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VarSymbol")
            .field("path", &self.path)
            .field("assigned_type", &self.assigned_type.as_deref())
            .finish()
    }
}
