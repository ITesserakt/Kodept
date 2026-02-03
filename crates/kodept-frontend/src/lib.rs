extern crate core;

use std::fmt::{Display, Formatter};
use std::ops::ControlFlow;

pub mod engine;
mod read_code_source;
mod report;
mod source_files;
mod traits;

pub mod prelude {
    pub use super::read_code_source::{ReadSource, Source, TryReadCode};
    pub use super::report::{ExtractReports, Global};
    pub use super::source_files::{CollectedSources, SourceFiles, SourceView};
    pub use super::traits::{Compiler, Interpreter};
}

#[derive(Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Display for Either<A, B>
where
    A: Display,
    B: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::Left(x) => x.fmt(f),
            Either::Right(x) => x.fmt(f),
        }
    }
}

/// Some execution that can break with
/// failure (and this failure got reported)
/// or continue with value [T].
pub type Execution<T> = ControlFlow<(), T>;
