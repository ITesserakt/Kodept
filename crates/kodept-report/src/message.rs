use crate::traits::{ad_hoc_message, IntoSpannedReportMessage, SpannedReportMessage};
use crate::Str;
use kodept_core::code_point::{CodePoint, Span};
use std::borrow::Cow;
use std::error::Error;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    pub(crate) point: Span,
    pub(crate) primary: bool,
    pub(crate) message: Str,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    Bug,
    Error,
    Warning,
    Note,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    pub(crate) message: Str,
    pub(crate) labels: Vec<Label>,
    pub(crate) notes: Vec<Str>,
    pub(crate) severity: Severity,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReportMessage {
    severity: Severity,
    notes: Vec<Str>,
    message: Str,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpannedError<E> {
    point: Span,
    severity: Severity,
    notes: Vec<Str>,
    inner: E,
}

impl Severity {
    pub(crate) fn into_codespan(self) -> codespan_reporting::diagnostic::Severity {
        use codespan_reporting::diagnostic::Severity as CSeverity;

        match self {
            Severity::Bug => CSeverity::Bug,
            Severity::Error => CSeverity::Error,
            Severity::Warning => CSeverity::Warning,
            Severity::Note => CSeverity::Note,
        }
    }
}

impl Label {
    pub fn primary(message: impl Into<Str>, at: impl Into<Span>) -> Self {
        Self {
            point: at.into(),
            primary: true,
            message: message.into(),
        }
    }

    pub fn secondary(message: impl Into<Str>, at: impl Into<Span>) -> Self {
        Self {
            point: at.into(),
            primary: false,
            message: message.into(),
        }
    }
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

    pub fn with_message(self, message: impl Into<Str>) -> Self {
        Self {
            message: message.into(),
            ..self
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_primary_label(mut self, message: impl Into<Str>, at: impl Into<Span>) -> Self {
        self.labels.push(Label::primary(message, at));
        self
    }

    pub fn with_secondary_label(mut self, message: impl Into<Str>, at: impl Into<Span>) -> Self {
        self.labels.push(Label::secondary(message, at));
        self
    }

    pub fn with_note(mut self, note: impl Into<Str>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl ReportMessage {
    pub fn new(severity: Severity, message: impl Into<Str>) -> Self {
        Self {
            severity,
            message: message.into(),
            notes: Default::default(),
        }
    }

    pub fn with_note(mut self, note: Cow<'static, str>) -> Self {
        self.notes.push(note);
        self
    }
}

impl<E: Error> SpannedError<E> {
    pub fn new(inner: E, at: impl Into<Span>) -> Self {
        Self {
            point: at.into(),
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

    pub fn map<F: Error>(self, f: impl FnOnce(E) -> F) -> SpannedError<F> {
        SpannedError {
            point: self.point,
            severity: self.severity,
            notes: self.notes,
            inner: f(self.inner),
        }
    }
}

impl From<ReportMessage> for Diagnostic {
    fn from(value: ReportMessage) -> Self {
        let mut this = Self::new(value.severity);
        this.message = value.message;
        this.notes = value.notes;
        this
    }
}

impl<E: Error> From<SpannedError<E>> for Diagnostic {
    fn from(value: SpannedError<E>) -> Self {
        let mut this = Self::new(value.severity);
        this.labels.push(Label::primary("here", value.point));
        this.message = value.inner.to_string().into();
        this.notes = value.notes;
        this
    }
}

impl SpannedReportMessage for ReportMessage {
    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage {
        ad_hoc_message(move || {
            let mut diagnostic = Diagnostic::from(self);
            diagnostic
                .labels
                .push(Label::secondary("while checking", location));
            diagnostic
        })
    }
}

impl SpannedReportMessage for Diagnostic {
    fn with_node_location(mut self, location: CodePoint) -> impl IntoSpannedReportMessage {
        ad_hoc_message(move || {
            self.labels
                .push(Label::secondary("while checking", location));
            self
        })
    }
}

impl<E: Error> SpannedReportMessage for SpannedError<E> {
    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage {
        ad_hoc_message(move || {
            let mut diagnostic = Diagnostic::from(self);
            diagnostic
                .labels
                .push(Label::secondary("while checking", location));
            diagnostic
        })
    }
}
