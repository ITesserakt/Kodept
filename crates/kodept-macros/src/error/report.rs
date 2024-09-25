use codespan_reporting::diagnostic::{Diagnostic, Label as ForeignLabel};
use kodept_core::code_point::CodePoint;
use std::any::{type_name_of_val, TypeId};
use std::borrow::Cow;
use std::hash::{DefaultHasher, Hash, Hasher};
use crate::context::FileId;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Severity {
    Bug,
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone)]
pub struct Label {
    point: CodePoint,
    primary: bool,
    message: Cow<'static, str>,
}

#[derive(Debug)]
pub struct ReportMessage {
    severity: Severity,
    notes: Vec<Cow<'static, str>>,
    message: String,
}

pub trait SpannedReportMessage {
    fn labels(&self) -> impl IntoIterator<Item = Label>;
    fn severity(&self) -> Severity;
    fn message(&self) -> Cow<'static, str>;
    fn notes(&self) -> impl IntoIterator<Item = Cow<'static, str>>;

    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage;
}

pub trait IntoSpannedReportMessage {
    type Message: SpannedReportMessage + 'static;

    fn into_message(self) -> Self::Message;
}

#[derive(Debug)]
pub struct Report<FileId = crate::context::FileId> {
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
    pub fn new<S: Into<String>>(severity: Severity, _: S, message: String) -> Self {
        Self {
            severity,
            message,
            notes: Default::default(),
        }
    }

    pub fn with_note(mut self, note: Cow<'static, str>) -> Self {
        self.notes.push(note);
        self
    }
}

impl<FileId> Report<FileId> {
    fn generate_code_from_type<T>(value: &T) -> String {
        let type_name = type_name_of_val(value);
        let mut hasher = DefaultHasher::new();
        type_name.hash(&mut hasher);
        let hash = hasher.finish();
        let hash = hash.to_ne_bytes().into_iter().fold(0u16, |acc, next| {
            acc ^ next as u16
        });
        format!("{:0>8X}", hash)
    }

    fn from_raw_message_with_code<T>(file_id: FileId, msg: T, code: String) -> Self
    where T: SpannedReportMessage,
        FileId: Clone
    {
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
            .with_code(code)
            .with_notes(msg.notes().into_iter().map(|it| it.to_string()).collect())
            .with_labels(labels);

        Self { diagnostic }
    }

    #[must_use]
    pub fn from_message<T>(file_id: FileId, msg: T) -> Self
    where
        T: IntoSpannedReportMessage,
        FileId: Clone,
    {
        let code = Self::generate_code_from_type(&msg);
        Self::from_raw_message_with_code(file_id, msg.into_message(), code)
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        use codespan_reporting::diagnostic::Severity as ForeignSeverity;

        matches!(
            self.diagnostic.severity,
            ForeignSeverity::Error | ForeignSeverity::Bug
        )
    }

    #[must_use]
    pub(crate) fn into_diagnostic(self) -> Diagnostic<FileId> {
        self.diagnostic
    }
}

impl<E: std::error::Error> From<E> for ReportMessage {
    fn from(value: E) -> Self {
        ReportMessage {
            severity: Severity::Error,
            message: value.to_string(),
            notes: Default::default(),
        }
    }
}

impl<T: Into<ReportMessage>> IntoSpannedReportMessage for T {
    type Message = ReportMessage;

    fn into_message(self) -> Self::Message {
        self.into()
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

    fn notes(&self) -> impl IntoIterator<Item = Cow<'static, str>> {
        self.notes.clone()
    }

    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage {
        crate::error::Diagnostic::new(self.severity)
            .with_message(self.message)
            .with_node_location(location)
    }
}
