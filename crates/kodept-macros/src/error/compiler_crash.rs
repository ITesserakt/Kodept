use codespan_reporting::diagnostic::Severity;

use crate::error::report::ReportMessage;

pub struct CompilerCrash {
    message: String,
}

impl From<CompilerCrash> for ReportMessage {
    fn from(value: CompilerCrash) -> Self {
        ReportMessage::new(Severity::Bug, "KC666", value.message)
    }
}
