use crate::error::report::ReportMessage;
use codespan_reporting::diagnostic::Severity;

pub struct CompilerCrash {
    message: String,
}

impl From<CompilerCrash> for ReportMessage {
    fn from(value: CompilerCrash) -> Self {
        ReportMessage::new(Severity::Bug, "KC666", value.message)
    }
}
