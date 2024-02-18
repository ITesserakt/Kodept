pub use codespan_reporting::diagnostic::Severity;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use extend::ext;
use kodept_ast::rlt_accessor::RLTFamily;
use kodept_ast::traits::IntoASTFamily;

use crate::traits::Context;
use crate::warn_about_broken_rlt;
use kodept_core::code_point::CodePoint;
use kodept_core::file_relative::CodePath;
use kodept_core::ConvertibleToRef;

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
            let mut secondaries: Vec<_> = s
                .iter()
                .map(|it| Label::secondary((), it.as_range()))
                .collect();
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

#[ext]
pub impl<T: Default, E: Into<ReportMessage>> Result<T, E> {
    fn report_errors<F, U>(self, at: &impl IntoASTFamily, context: &mut impl Context, func: F) -> T
    where
        RLTFamily: ConvertibleToRef<U>,
        F: Fn(&U) -> Vec<CodePoint>,
    {
        match self {
            Ok(x) => x,
            Err(error) => {
                let points = context.access(at).map_or_else(
                    || {
                        warn_about_broken_rlt::<U>();
                        vec![]
                    },
                    func,
                );
                context.add_report(points, error);
                T::default()
            }
        }
    }
}
