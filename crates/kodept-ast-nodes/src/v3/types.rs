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
        name: Option<Str>,
        ty_id: T,
    },
    Named {
        name: Str,
        ty_id: T,
        default_expr_id: Option<NodeId>,
    },
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct ValueCtor<T> {
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
pub struct UserFunction<T> {
    pub params: Vec<Param<T>>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
#[require(Node::of::<Self>())]
pub struct ForeignFunction<T> {
    pub params: Vec<T>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct AnonFunction<T> {
    pub params: Vec<Param<T>>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Variable<T> {
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
pub struct Value<T> {
    pub inner: T,
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

pub(super) trait NameRef: Send + Sync + 'static {}
pub(super) trait TypeRef<const REQUIRED: bool>: Send + Sync + 'static {}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub is_global: bool,
    pub segments: Cow<'static, [Str]>,
}

#[derive(Debug, PartialEq)]
pub enum VariableName {
    Empty,
    Name(Str),
}

#[derive(Debug, PartialEq)]
pub enum Unresolved {
    Named { context: Path, ident: Str },
    Tuple(Vec<Unresolved>),
}

#[derive(Debug, PartialEq)]
pub struct Resolved(pub NodeId);

impl NameRef for Unresolved {}
impl NameRef for Resolved {}

impl TypeRef<true> for Unresolved {}
impl<const REQUIRED: bool> TypeRef<REQUIRED> for Resolved {}

impl TypeRef<false> for Option<Unresolved> {}

impl Path {
    pub const fn empty(is_global: bool) -> Self {
        Self {
            is_global,
            segments: Cow::Borrowed(&[]),
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
