mod source_files;
mod read_code_source;

pub mod prelude {
    pub use super::source_files::{SourceFiles, SourceView, GlobalReports};
    pub use super::read_code_source::{TryReadCode, ReadSource, Source};
}
