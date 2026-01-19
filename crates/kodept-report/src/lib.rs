//! This crate contains an abstraction for compiler diagnostics - `reports`.

use std::borrow::Cow;

pub mod codespan;
pub mod crash;
pub mod files;
pub mod message;
pub mod report;
pub mod traits;

type Str = Cow<'static, str>;

pub use kodept_core::file_name::{FileDescriptor, FileId};

pub mod prelude {
    pub use super::codespan::{CodespanSettings, Reportable, Settings};
    pub use super::message::{Diagnostic, Label, ReportMessage, Severity, SpannedError};
    pub use super::report::Report;
    pub use super::traits::{
        IntoSpannedReportMessage, MessageBehaviour, SpannedReportMessage, ad_hoc_message,
    };
}
