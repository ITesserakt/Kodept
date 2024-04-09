#![feature(iter_intersperse)]

use kodept_macros::error::report::{ReportMessage, Severity};

use crate::scope::ScopeError;

mod convert_model;
pub mod operator_desugaring;
mod scope;
pub mod semantic_analyzer;
mod symbol;
pub mod type_checker;
mod utils;

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
