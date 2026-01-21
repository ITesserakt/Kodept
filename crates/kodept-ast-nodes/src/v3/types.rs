use crate::term::ReferenceContext;
use bevy_ecs::prelude::Component;
use bevy_ecs::prelude::{Entity, Name};
use bigdecimal::BigDecimal;
use kodept_ast::Str;
use num_bigint::BigInt;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
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
        default_expr_id: Option<Entity>,
    },
}

#[derive(Debug, PartialEq, Component)]
pub struct TypeCtor<T> {
    pub name: CtorName,
    pub params: Vec<Param<T>>,
}

#[derive(Debug, PartialEq, Component)]
pub struct UserType;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct PrimType;

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct UserFunction<T> {
    pub params: Vec<Param<T>>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct ForeignFunction<T> {
    pub params: Vec<T>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
pub struct AnonFunction<T> {
    pub params: Vec<Param<T>>,
    pub return_type: T,
}

#[derive(Debug, PartialEq, Component)]
#[require(Name)]
pub struct Variable<T> {
    pub mutable: bool,
    pub annotation: T,
}

#[derive(Debug, PartialEq, Component)]
pub struct Block;

#[derive(Debug, PartialEq, Component)]
pub struct Value<T> {
    pub inner: T,
}

#[derive(Debug, PartialEq, Component)]
pub enum Literal {
    Integer(BigInt),
    Floating(BigDecimal),
    Char(char),
    String(Str),
}

#[derive(Debug, PartialEq, Component)]
pub struct Tuple;

#[derive(Debug, PartialEq, Component)]
pub struct Call;

#[derive(Debug, PartialEq, Component)]
pub struct If;

#[derive(Debug, PartialEq, Component)]
pub struct Branch;

#[derive(Debug, PartialEq, Component)]
pub struct Otherwise;

pub(super) trait NameRef: Send + Sync + 'static {}
pub(super) trait TypeRef<const REQUIRED: bool>: Send + Sync + 'static {}

#[derive(Debug, PartialEq)]
pub enum Unresolved {
    Named {
        context: ReferenceContext,
        ident: Str,
    },
    Tuple(Vec<Unresolved>),
}

#[derive(Debug, PartialEq)]
pub struct Resolved(pub Entity);

impl NameRef for Unresolved {}
impl NameRef for Resolved {}

impl TypeRef<true> for Unresolved {}
impl<const REQUIRED: bool> TypeRef<REQUIRED> for Resolved {}

impl TypeRef<false> for Option<Unresolved> {}
