use base64::alphabet::CRYPT;
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::Engine;
use codespan_reporting::diagnostic::{Diagnostic, Label as ForeignLabel};
use kodept_core::code_point::CodePoint;
use kodept_core::file_name::FileName;
use std::any::TypeId;
use std::borrow::Cow;
use std::convert::Infallible;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Severity {
    Bug,
    Error,
    Warning,
    Note,
}

#[derive(Debug)]
pub struct Label {
    point: CodePoint,
    primary: bool,
    message: Cow<'static, str>,
}

#[derive(Debug)]
pub struct ReportMessage {
    pub severity: Severity,
    pub code: String,
    pub message: String,
}

pub trait SpannedReportMessage {
    fn labels(&self) -> impl IntoIterator<Item = Label>;
    fn severity(&self) -> Severity;
    fn message(&self) -> Cow<'static, str>;
}

pub trait IntoSpannedReportMessage {
    type Message: SpannedReportMessage + 'static;

    fn into_message(self) -> Self::Message;
}

#[derive(Debug)]
pub struct Report<FileId = ()> {
    diagnostic: Diagnostic<FileId>,
}

impl Label {
    pub fn primary(message: impl Into<Cow<'static, str>>, at: CodePoint) -> Self {
        Self {
            point: at,
            primary: true,
            message: message.into(),
        }
    }

    pub fn secondary(message: impl Into<Cow<'static, str>>, at: CodePoint) -> Self {
        Self {
            point: at,
            primary: false,
            message: message.into(),
        }
    }
}

impl From<Severity> for codespan_reporting::diagnostic::Severity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Bug => Self::Bug,
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Note => Self::Note,
        }
    }
}

impl ReportMessage {
    pub fn new<S: Into<String>>(severity: Severity, code: S, message: String) -> Self {
        Self {
            severity,
            code: code.into(),
            message,
        }
    }
}

impl<FileId> Report<FileId> {
    fn generate_code_from_type<T: 'static + ?Sized>() -> String {
        let type_id = TypeId::of::<T>();
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        let hash = hasher.finish();
        let bytes = hash.to_le_bytes();
        GeneralPurpose::new(&CRYPT, GeneralPurposeConfig::new()).encode(bytes)
    }

    pub fn from_message<T>(file_id: FileId, msg: T) -> Self
    where
        T: IntoSpannedReportMessage,
        FileId: Clone,
    {
        let msg = msg.into_message();
        let labels = msg
            .labels()
            .into_iter()
            .map(|it| {
                let label = if it.primary {
                    ForeignLabel::primary(file_id.clone(), it.point.as_range())
                } else {
                    ForeignLabel::secondary(file_id.clone(), it.point.as_range())
                };
                label.with_message(it.message)
            })
            .collect();

        let diagnostic = Diagnostic::new(msg.severity().into())
            .with_message(msg.message())
            .with_code(Self::generate_code_from_type::<T::Message>())
            .with_labels(labels);

        Self { diagnostic }
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        use codespan_reporting::diagnostic::Severity as ForeignSeverity;

        matches!(
            self.diagnostic.severity,
            ForeignSeverity::Error | ForeignSeverity::Bug
        )
    }

    pub fn into_diagnostic(self) -> Diagnostic<FileId> {
        self.diagnostic
    }
}

impl<FileId: Default> Report<FileId> {
    pub fn new<R: Into<ReportMessage>>(
        _file: &FileName,
        points: Vec<CodePoint>,
        message: R,
    ) -> Self {
        let msg = message.into();
        let diagnostic = Diagnostic::new(msg.severity.into())
            .with_code(msg.code)
            .with_message(msg.message);
        let diagnostic = if let [p] = points.as_slice() {
            diagnostic.with_labels(vec![ForeignLabel::primary(FileId::default(), p.as_range())])
        } else if let [p, s @ ..] = points.as_slice() {
            let mut secondaries: Vec<_> = s
                .iter()
                .map(|it| ForeignLabel::secondary(FileId::default(), it.as_range()))
                .collect();
            secondaries.insert(0, ForeignLabel::primary(FileId::default(), p.as_range()));
            diagnostic.with_labels(secondaries)
        } else {
            diagnostic
        };
        Self { diagnostic }
    }
}

impl From<Infallible> for ReportMessage {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl<T: Into<ReportMessage>> IntoSpannedReportMessage for T {
    type Message = ReportMessage;

    fn into_message(self) -> Self::Message {
        self.into()
    }
}
