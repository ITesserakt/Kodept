use crate::context::FileId;
use crate::error::report::{
    IntoSpannedReportMessage, Label, Report, ReportMessage, Severity, SpannedReportMessage,
};
use crate::error::ErrorReported;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::WriteColor;
use codespan_reporting::term::Config;
use extend::ext;
use kodept_ast::graph::{AnyNode, NodeId};
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub struct CodespanSettings<S> {
    pub config: Config,
    pub stream: S,
}

#[derive(Debug)]
pub struct SpannedError<E: std::error::Error> {
    point: CodePoint,
    severity: Severity,
    inner: E,
}

pub trait Reportable {
    type FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error>;
}

impl<FileId> Reportable for Diagnostic<FileId> {
    type FileId = FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        codespan_reporting::term::emit(&mut settings.stream, &settings.config, source, &self)
    }
}

impl<FileId> Reportable for Report<FileId> {
    type FileId = FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        self.into_diagnostic().emit(settings, source)
    }
}

impl<R: Reportable> Reportable for Vec<R> {
    type FileId = R::FileId;

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        self.into_iter()
            .try_for_each(|it| it.emit(settings, source))
    }
}

#[ext]
pub impl<T, R: Reportable> Result<T, R> {
    fn or_emit<'f, W: WriteColor, F: Files<'f, FileId = R::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<T, ErrorReported> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                e.emit(settings, source).expect("Cannot emit diagnostics");
                Err(ErrorReported::new())
            }
        }
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
                impl<'e, E> From<Helper<'e, E>> for ReportMessage
                where
                    E: std::error::Error,
                {
                    fn from(value: Helper<E>) -> Self {
                        Self::new(Severity::Error, "external", value.0.to_string())
                    }
                }

                let report = Report::from_message(file_id, Helper(&e));
                report
                    .emit(settings, source)
                    .expect("Cannot emit diagnostic");
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
            inner,
        }
    }

    pub fn for_node<T>(inner: E, node_id: NodeId<T>, ctx: &RLTAccessor) -> Self
    where
        AnyNode: TryFrom<T>,
    {
        let position = ctx.get_unknown(node_id);
        match position {
            None => panic!("Node is not linked with corresponding rlt node"),
            Some(pos) => Self::new(inner, pos.location()),
        }
    }

    pub fn with_severity(self, severity: Severity) -> Self {
        Self { severity, ..self }
    }
}

impl<E: std::error::Error> SpannedReportMessage for SpannedError<E> {
    fn labels(&self) -> impl IntoIterator<Item = Label> {
        [Label::primary("here", self.point)]
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Owned(self.inner.to_string())
    }
}

impl<E: std::error::Error> Display for SpannedError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SpannedError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl SpannedReportMessage for ReportMessage {
    fn labels(&self) -> impl IntoIterator<Item = Label> {
        []
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn message(&self) -> Cow<'static, str> {
        Cow::Owned(self.message.clone())
    }
}

impl<E: std::error::Error + 'static> IntoSpannedReportMessage for SpannedError<E> {
    type Message = Self;

    fn into_message(self) -> Self::Message {
        self
    }
}
