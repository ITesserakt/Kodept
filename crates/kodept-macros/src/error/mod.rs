use std::borrow::Cow;
use derive_more::Display;
use std::error::Error;
use std::fmt::Formatter;
use kodept_core::code_point::CodePoint;
use crate::error::report::{IntoSpannedReportMessage, Label, Severity, SpannedReportMessage};

pub mod compiler_crash;
pub mod report;
pub mod report_collector;
pub mod traits;

#[derive(Debug, Default)]
pub struct ErrorReported {
    cause: Option<anyhow::Error>,
}

impl ErrorReported {
    pub fn with_cause<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self {
            cause: Some(anyhow::Error::from(error)),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl Error for ErrorReported {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.cause {
            None => None,
            Some(x) => Some(x.as_ref()),
        }
    }
}

impl Display for ErrorReported {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compilation failed due to produced errors")
    }
}

pub struct Diagnostic {
    message: Cow<'static, str>,
    labels: Vec<Label>,
    notes: Vec<Cow<'static, str>>,
    severity: Severity
}

impl Diagnostic {
    pub fn new(severity: Severity) -> Self {
        Self {
            message: Default::default(),
            labels: Default::default(),
            notes: Default::default(),
            severity,
        }
    }

    pub fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
            ..self
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_note(mut self, note: Cow<'static, str>) -> Self {
        self.notes.push(note);
        self
    }
}

impl IntoSpannedReportMessage for Diagnostic {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        self
    }
}

impl SpannedReportMessage for Diagnostic {
    fn labels(&self) -> impl IntoIterator<Item=Label> {
        self.labels.clone()
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> Cow<'static, str> {
        self.message.clone()
    }

    fn notes(&self) -> impl IntoIterator<Item=Cow<'static, str>> {
        self.notes.clone()
    }

    fn with_node_location(mut self, location: CodePoint) -> impl IntoSpannedReportMessage {
        self.labels.push(Label::secondary("while checking here", location));
        self
    }
}
