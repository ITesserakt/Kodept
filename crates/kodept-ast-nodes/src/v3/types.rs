use bigdecimal::BigDecimal;
use kodept_ast::Str;
use kodept_ast::prelude::{CodeHolder, NodeId};
use kodept_ast::properties::{Name, Node, NodeProperty, RequireProperty};
use kodept_ecs::component::Component;
use kodept_ecs::exported::bevy_ecs;
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

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
#[require(Name)]
pub struct Param;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct ValueCtor {
    pub name: CtorName,
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
pub struct UserFunction;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
#[require(Node::of::<Self>())]
pub struct ForeignFunction;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct AnonFunction;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Variable {
    pub mutable: bool,
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

#[derive(Debug, PartialEq, Component)]
pub enum TypeAnnotation {
    Infer,
    Named { path: Path, ident: Str },
    Tuple(Vec<Self>),
}

#[derive(Debug, PartialEq, Component)]
pub enum UnresolvedType {
    Named { path: Path, ident: Str },
    Tuple(Vec<Self>),
}

#[derive(Debug, PartialEq, Component)]
pub enum ResolvedTypeAnnotation {
    Infer,
    Named(NodeId),
    Tuple(Vec<Self>),
}

#[derive(Debug, PartialEq, Component)]
pub enum ResolvedType {
    Named(NodeId),
    Tuple(Vec<Self>),
}

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

impl NodeProperty for TypeAnnotation {}
impl NodeProperty for UnresolvedType {}
impl NodeProperty for ResolvedTypeAnnotation {}
impl NodeProperty for ResolvedType {}

impl RequireProperty<TypeAnnotation> for Param {}
impl RequireProperty<ResolvedTypeAnnotation> for Param {}
impl RequireProperty<UnresolvedType> for Param {}
impl RequireProperty<ResolvedType> for Param {}

impl RequireProperty<TypeAnnotation> for Variable {}
impl RequireProperty<ResolvedTypeAnnotation> for Variable {}

impl RequireProperty<TypeAnnotation> for UserFunction {}
impl RequireProperty<ResolvedTypeAnnotation> for UserFunction {}

impl RequireProperty<UnresolvedType> for ForeignFunction {}
impl RequireProperty<ResolvedType> for ForeignFunction {}

impl RequireProperty<TypeAnnotation> for AnonFunction {}
impl RequireProperty<ResolvedTypeAnnotation> for AnonFunction {}
