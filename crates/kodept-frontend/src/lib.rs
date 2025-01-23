use std::ops::ControlFlow;

mod read_code_source;
mod source_files;
mod report;

pub mod prelude {
    pub use super::read_code_source::{ReadSource, Source, TryReadCode};
    pub use super::source_files::{SourceFiles, SourceView};
    pub use super::report::{GlobalReports, Reports, Global};
}

/// Some execution that can break with 
/// failure (and this failure got reported) 
/// or continue with value [T].
pub type Execution<T> = ControlFlow<(), T>;

