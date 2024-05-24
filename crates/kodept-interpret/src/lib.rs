#![feature(iter_intersperse)]

use std::marker::PhantomData;

use kodept_macros::error::report::{ReportMessage, Severity};

use crate::operator_desugaring::{AccessExpander, BinaryOperatorExpander, UnaryOperatorExpander};
use crate::scope::ScopeError;

mod convert_model;
mod node_family;
pub mod operator_desugaring;
mod scope;
pub mod semantic_analyzer;
mod symbol;
pub mod type_checker;
// mod typing;
mod utils;

#[derive(Copy, Clone)]
pub struct Witness(PhantomData<()>);

impl Witness {
    pub fn fact(_: AccessExpander, _: BinaryOperatorExpander, _: UnaryOperatorExpander) -> Witness {
        Witness(PhantomData)
    }

    pub fn prove<T>(self) -> T {
        panic!("Cannot prove contract")
    }
}

pub enum Errors {
    UnresolvedReference(Path),
    AlreadyDefined(Path),
    TooComplex,
    Scope(ScopeError),
}

impl From<Errors> for ReportMessage {
    fn from(value: Errors) -> Self {
        match value {
            Errors::UnresolvedReference(name) => ReportMessage::new(
                Severity::Error,
                "SM001",
                format!("Cannot resolve reference `{name}`"),
            ),
            Errors::AlreadyDefined(name) => ReportMessage::new(
                Severity::Error,
                "SM002",
                format!("`{name}` already defined"),
            ),
            Errors::Scope(inner) => ReportMessage::new(Severity::Bug, "SM003", inner.to_string()),
            Errors::TooComplex => ReportMessage::new(
                Severity::Bug,
                "SM004",
                "Complex types is not supported yet".to_string(),
            ),
        }
    }
}

pub(crate) type Path = String;
