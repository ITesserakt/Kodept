use crate::error::report::{IntoSpannedReportMessage, Label, Report, ReportMessage, Severity, SpannedReportMessage};
use crate::error::report_collector::{ReportCollector, Reporter};
use crate::error::{Diagnostic, ErrorReported};
use crate::FileId;
use codespan_reporting::files::Files;
use codespan_reporting::term::termcolor::WriteColor;
use codespan_reporting::term::Config;
use extend::ext;
use kodept_core::code_point::CodePoint;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug)]
pub struct CodespanSettings<S> {
    pub config: Config,
    pub stream: S,
}

#[derive(Debug)]
pub struct SpannedError<E> {
    point: CodePoint,
    severity: Severity,
    notes: Vec<Cow<'static, str>>,
    inner: E,
}

pub trait DrainReports {
    type Output;

    fn drain(self, file_id: FileId, collector: &mut ReportCollector) -> Self::Output;
}

pub trait Reportable {
    type FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    );
}

impl<FileId> Reportable for Report<FileId> {
    type FileId = FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) {
        codespan_reporting::term::emit(
            &mut settings.stream,
            &settings.config,
            source,
            &self.into_diagnostic(),
        )
        .expect("Cannot emit diagnostics")
    }
}

impl<R: Reportable> Reportable for Vec<R> {
    type FileId = R::FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) {
        self.into_iter().for_each(|it| it.emit(settings, source))
    }
}

#[ext]
pub impl<T, E: std::error::Error + Send + Sync + 'static> Result<T, E> {
    fn or_emit<'f, W: WriteColor, F: Files<'f, FileId = FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
        file_id: FileId,
    ) -> Result<T, ErrorReported> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                pub struct Helper<'e, E: std::error::Error>(&'e E);
                impl<'e, E: std::error::Error> IntoSpannedReportMessage for Helper<'e, E> {
                    type Message = ReportMessage;

                    fn into_message(self) -> Self::Message {
                        ReportMessage::new(Severity::Error, self.0.to_string())
                    }
                }

                Report::from_message(file_id, Helper(&e)).emit(settings, source);
                Err(ErrorReported::with_cause(e))
            }
        }
    }
}

impl<E: std::error::Error> SpannedError<E> {
    pub fn new(inner: E, at: CodePoint) -> Self {
        Self {
            point: at,
            severity: Severity::Error,
            notes: Default::default(),
            inner,
        }
    }

    pub fn with_severity(self, severity: Severity) -> Self {
        Self { severity, ..self }
    }

    pub fn with_note(mut self, note: impl Into<Cow<'static, str>>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn map<F: std::error::Error>(self, f: impl FnOnce(E) -> F) -> SpannedError<F> {
        SpannedError {
            point: self.point,
            severity: self.severity,
            notes: self.notes,
            inner: f(self.inner),
        }
    }
}

impl<E: ToString> From<SpannedError<E>> for Diagnostic {
    fn from(val: SpannedError<E>) -> Self {
        Diagnostic {
            message: Cow::Owned(val.inner.to_string()),
            labels: vec![Label::primary("here", val.point)],
            notes: val.notes,
            severity: val.severity,
        }
    }
}

impl<E: ToString> SpannedReportMessage for SpannedError<E> {
    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage {
        Diagnostic::new(self.severity)
            .with_message(self.inner.to_string())
            .with_label(Label::primary("here", self.point))
            .with_node_location(location)
    }
}

impl<E: std::error::Error> Display for SpannedError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<E: std::error::Error + 'static> IntoSpannedReportMessage for SpannedError<E> {
    type Message = Self;

    fn into_message(self) -> Self::Message {
        self
    }
}

impl<T, S, I> DrainReports for Result<T, I>
where
    S: IntoSpannedReportMessage,
    I: IntoIterator<Item = S>,
{
    type Output = Option<T>;

    fn drain(self, file_id: FileId, collector: &mut ReportCollector) -> Self::Output {
        match self {
            Ok(x) => Some(x),
            Err(e) => {
                for item in e {
                    collector.report(file_id, item);
                }
                None
            }
        }
    }
}
