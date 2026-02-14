use bigdecimal::BigDecimal;
use kodept_ast::Str;
use kodept_ast::export::Component;
use kodept_ast::export::bevy_ecs;
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Name, Node};
use kodept_rlt::prelude::Context;
use num_bigint::BigInt;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Modules;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
#[require(Node::of::<Self>())]
pub struct Module;

#[derive(Debug, PartialEq)]
pub enum CtorName {
    Inline,
    Explicit(Str),
}

#[derive(Debug, PartialEq)]
pub enum Param<T> {
    Positional {
        name: Str,
        ty: T,
    },
    Named {
        name: Str,
        ty: T,
        default_expr_id: Option<NodeId>,
    },
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct ValueCtor<T: TypeRef<true>> {
    pub name: CtorName,
    pub params: Vec<Param<T>>,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct UserType;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
#[require(Node::of::<Self>())]
pub struct PrimType;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
#[require(Node::of::<Self>())]
pub struct UserFunction<T: TypeRef<false>> {
    pub params: Vec<Param<T>>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
#[require(Node::of::<Self>())]
pub struct ForeignFunction<T: TypeRef<true>> {
    pub params: Vec<T>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct AnonFunction<T: TypeRef<false>> {
    pub params: Vec<Param<T>>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Variable<T: TypeRef<false>> {
    pub mutable: bool,
    pub annotation: T,
    pub name: VariableName,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Block<const NORMALIZED: bool = false>;
pub type NormalizedBlock = Block<true>;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Value {
    pub path: Path,
    pub ident: Str,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub enum Literal {
    Integer(BigInt),
    Floating(BigDecimal),
    Char(char),
    String(Str),
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Tuple;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Call;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct If;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Branch;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Otherwise;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Link;

pub trait TypeRef<const REQUIRED: bool>: Send + Sync + 'static {}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub is_global: bool,
    pub segments: Cow<'static, [Str]>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VariableName {
    Empty,
    Name(Str),
}

#[derive(Debug, PartialEq)]
pub enum TypeAnnotation {
    Infer,
    Named { path: Path, ident: Str },
    Tuple(Vec<Self>),
}

#[derive(Debug, PartialEq)]
pub enum UnresolvedType {
    Named { path: Path, ident: Str },
    Tuple(Vec<Self>),
}

#[derive(Debug, PartialEq)]
pub enum ResolvedTypeAnnotation {
    Infer,
    Named(NodeId),
    Tuple(Vec<Self>),
}

#[derive(Debug, PartialEq)]
pub enum ResolvedType {
    Named(NodeId),
    Tuple(Vec<Self>),
}

impl TypeRef<true> for UnresolvedType {}
impl TypeRef<true> for ResolvedType {}
impl TypeRef<false> for TypeAnnotation {}
impl TypeRef<false> for ResolvedTypeAnnotation {}

impl Path {
    pub const fn empty(is_global: bool) -> Self {
        Self {
            is_global,
            segments: Cow::Borrowed(&[]),
        }
    }
}

impl VariableName {
    pub fn as_str(&self) -> &str {
        match self {
            VariableName::Empty => "_",
            VariableName::Name(name) => name,
        }
    }
}

impl<T> Param<T> {
    pub fn name(&self) -> &str {
        match self {
            Param::Positional { name, .. } => name,
            Param::Named { name, .. } => name,
        }
    }

    pub fn ty(&self) -> &T {
        match self {
            Param::Positional { ty, .. } => ty,
            Param::Named { ty, .. } => ty,
        }
    }
}

impl UnresolvedType {
    pub fn into_annotation(self) -> TypeAnnotation {
        match self {
            UnresolvedType::Named { path, ident } => TypeAnnotation::Named { path, ident },
            UnresolvedType::Tuple(items) => {
                TypeAnnotation::Tuple(items.into_iter().map(|it| it.into_annotation()).collect())
            }
        }
    }
}

impl<T: CodeHolder> From<(&Context, T)> for Path {
    fn from((value, source): (&Context, T)) -> Self {
        let (is_global, items) = value.unfold();
        Path {
            is_global: is_global.is_some(),
            segments: items
                .into_iter()
                .map(|it| source.get_chunk_located(it))
                .collect(),
        }
    }
}
