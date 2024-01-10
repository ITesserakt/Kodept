use codespan_reporting::diagnostic::Severity;
use derive_more::Constructor;
use std::any::Any;

use crate::error::report::ReportMessage;

#[derive(Constructor)]
pub struct CompilerCrash {
    message: Box<dyn Any + Send>,
}

impl From<CompilerCrash> for ReportMessage {
    fn from(value: CompilerCrash) -> Self {
        let message: String = if let Some(s) = value.message.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = value.message.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic happened".to_string()
        };
        ReportMessage::new(Severity::Bug, "KC666", message)
    }
}
