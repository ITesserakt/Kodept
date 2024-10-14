use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::Path;
use derive_more::{From, TryInto};
use kodept_ast::graph::{AnyNodeId, Identifiable};
use kodept_ast::interning::SharedStr;
use kodept_ast::{Identifier, Ref, ReferenceContext};
use kodept_inference::r#type::PolymorphicType;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum SymbolKind {
    Type,
    Variable,
    Parameter,
    Constant,
    Function
}

#[derive(Debug)]
pub struct SymbolV2<Type = Option<PolymorphicType>> {
    ast_node: AnyNodeId,
    context: ReferenceContext,
    ident: SharedStr,
    kind: SymbolKind,
    ty: Type,
}

impl SymbolV2 {
    pub fn from_ref(value: &Ref, kind: SymbolKind) -> Self {
        Self {
            ast_node: value.get_id().widen(),
            context: value.context.clone(),
            ident: match &value.ident {
                Identifier::TypeReference { name } => name.clone(),
                Identifier::Reference { name } => name.clone(),
            },
            kind,
            ty: None,
        }
    }
    
    pub fn new(node_id: AnyNodeId, context: ReferenceContext, ident: SharedStr, kind: SymbolKind) -> Self {
        Self {
            ast_node: node_id,
            context,
            ident,
            kind,
            ty: None,
        }
    }
    
    pub fn with_type(mut self, ty: impl Into<PolymorphicType>) -> Self {
        self.ty = Some(ty.into());
        self
    }
}

impl<T> PartialEq for SymbolV2<T> {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context && self.ident == other.ident && self.kind == other.kind
    }
}

impl<T> Eq for SymbolV2<T> {}

impl<T> PartialOrd for SymbolV2<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.context
                .partial_cmp(&other.context)?
                .then(self.ident.partial_cmp(&other.ident)?)
                .then(self.kind.partial_cmp(&other.kind)?),
        )
    }
}

impl<T> Ord for SymbolV2<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.context
            .cmp(&other.context)
            .then_with(|| self.ident.cmp(&other.ident))
            .then_with(|| self.kind.cmp(&other.kind))
    }
}

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
    pub path: Path,
    pub assigned_type: Option<Rc<TypeSymbol>>,
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
