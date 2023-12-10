use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use itertools::Itertools;
use kodept_core::code_point::CodePoint;
use kodept_core::file_relative::CodePath;
#[cfg(feature = "size-of")]
use size_of::SizeOf;
use std::mem::size_of;

#[derive(Debug)]
pub struct ReportMessage {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub additional_message: String,
}

#[derive(Debug)]
pub struct Report {
    diagnostic: Diagnostic<()>,
}

impl ReportMessage {
    pub fn new<S: Into<String>>(severity: Severity, code: S, message: String) -> Self {
        Self {
            severity,
            code: code.into(),
            message,
            additional_message: "here".to_string(),
        }
    }

    #[must_use]
    pub fn with_additional_message(self, additional_message: String) -> Self {
        Self {
            additional_message,
            ..self
        }
    }
}

impl Report {
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.diagnostic.severity, Severity::Error | Severity::Bug)
    }

    pub fn new<R: Into<ReportMessage>>(
        _file: &CodePath,
        points: Vec<CodePoint>,
        message: R,
    ) -> Self {
        let msg = message.into();
        let diagnostic = Diagnostic::new(msg.severity)
            .with_code(msg.code)
            .with_message(msg.message);
        let diagnostic = if let [p] = points.as_slice() {
            diagnostic.with_labels(vec![Label::primary((), p.as_range())])
        } else if let [p, s @ ..] = points.as_slice() {
            let mut secondaries = s
                .iter()
                .map(|it| Label::secondary((), it.as_range()))
                .collect_vec();
            secondaries.insert(0, Label::primary((), p.as_range()));
            diagnostic.with_labels(secondaries)
        } else {
            diagnostic
        };
        Self { diagnostic }
    }

    pub fn into_diagnostic(self) -> Diagnostic<()> {
        self.diagnostic
    }
}

#[cfg(feature = "size-of")]
impl SizeOf for Report {
    fn size_of_children(&self, context: &mut size_of::Context) {
        context.add(1); // severity
        self.diagnostic.message.size_of_children(context);
        self.diagnostic.code.size_of_children(context);
        self.diagnostic.notes.size_of_children(context);
        context.add_vectorlike(
            self.diagnostic.labels.len(),
            self.diagnostic.labels.capacity(),
            size_of::<Label<()>>(),
        );
    }
}