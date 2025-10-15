use std::ops::ControlFlow;

mod read_code_source;
mod report;
mod source_files;
mod traits;
pub mod engine;

pub mod prelude {
    pub use super::read_code_source::{ReadSource, Source, TryReadCode};
    pub use super::report::{ExtractReports, Global};
    pub use super::source_files::{SourceFiles, SourceView, CollectedSources};
    pub use super::traits::{Compiler, Interpreter};
}

#[derive(Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

/// Some execution that can break with
/// failure (and this failure got reported)
/// or continue with value [T].
pub type Execution<T> = ControlFlow<(), T>;
