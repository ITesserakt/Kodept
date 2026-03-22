extern crate core;

use std::ops::ControlFlow;

pub mod engine;
mod read_code_source;
mod report;
mod source_files;
mod traits;

pub mod prelude {
    pub use super::read_code_source::{ReadSource, TryReadSource};
    pub use super::report::{ExtractReports, Global};
    pub use super::source_files::{CollectedSources, SourceFiles, SourceView};
    pub use super::traits::{Compiler, Interpreter};
}

/// Some execution that can break with
/// failure (and this failure got reported)
/// or continue with value [T].
pub type Execution<T> = ControlFlow<(), T>;
