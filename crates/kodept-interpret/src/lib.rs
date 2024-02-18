mod scope;
mod semantic_analyzer;
mod symbol;

use kodept_macros::error::report::{ReportMessage, Severity};
pub use semantic_analyzer::SemanticAnalyzer;

pub enum Errors {
    UnresolvedReference(Path),
    AlreadyDefined(Path),
    WrongScope { expected: Path, found: Path },
    TooComplex,
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
            Errors::WrongScope { expected, found } => ReportMessage::new(
                Severity::Bug,
                "SM003",
                format!("Expected exit at scope `{expected}`, but exited scope `{found:?}`"),
            ),
            Errors::TooComplex => ReportMessage::new(
                Severity::Bug,
                "SM004",
                "Complex types is not supported yet".to_string(),
            ),
        }
    }
}

pub(crate) type Path = String;
