//! This crate contains an abstraction for compiler diagnostics - `reports`.

use std::borrow::Cow;

pub mod message;
pub mod traits;
pub mod report;
pub mod codespan;
pub mod crash;
pub mod files;

type Str = Cow<'static, str>;

pub use kodept_core::file_name::{FileId, FileDescriptor};

pub mod prelude {
    pub use super::message::{ReportMessage, Severity, Diagnostic, Label, SpannedError};
    pub use super::traits::{ad_hoc_message, IntoSpannedReportMessage, SpannedReportMessage, MessageBehaviour};
    pub use super::report::Report;
    pub use super::codespan::{Reportable, CodespanSettings, Settings};
}
