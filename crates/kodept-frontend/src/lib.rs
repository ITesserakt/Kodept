use std::ops::ControlFlow;

mod read_code_source;
mod report;
mod source_files;
mod traits;

pub mod prelude {
    pub use super::read_code_source::{ReadSource, Source, TryReadCode};
    pub use super::report::{Global, GlobalReports, Reports, ExtractReports};
    pub use super::source_files::{SourceFiles, SourceView};
    pub use super::traits::{Compiler, Interpreter};
}

/// Some execution that can break with
/// failure (and this failure got reported)
/// or continue with value [T].
pub type Execution<T> = ControlFlow<(), T>;
