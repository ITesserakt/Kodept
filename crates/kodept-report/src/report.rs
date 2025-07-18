use crate::{
    message::{Diagnostic, Severity},
    traits::IntoSpannedReportMessage,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Report<FileId = crate::FileId> {
    file_id: FileId,
    diagnostic: Diagnostic,
    code: String,
}

impl<FileId> Report<FileId> {
    #[must_use]
    pub fn from_message<T>(file_id: FileId, msg: T) -> Self
    where
        T: IntoSpannedReportMessage,
    {
        let code = format!("{:0>8X}", msg.code());
        let diagnostic = msg.into_message();
        Self {
            file_id,
            diagnostic: diagnostic.into(),
            code,
        }
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.diagnostic.severity, Severity::Error)
    }

    pub(crate) fn into_inner(self) -> codespan_reporting::diagnostic::Diagnostic<FileId>
    where
        FileId: Clone,
    {
        use codespan_reporting::diagnostic::{
            Diagnostic as CDiagnostic, Label as CLabel, Severity as CSeverity,
        };

        CDiagnostic::new(match self.diagnostic.severity {
            Severity::Bug => CSeverity::Bug,
            Severity::Error => CSeverity::Error,
            Severity::Warning => CSeverity::Warning,
            Severity::Note => CSeverity::Note,
        })
        .with_code(self.code)
        .with_message(self.diagnostic.message)
        .with_labels_iter(self.diagnostic.labels.into_iter().map(|it| {
            match it.primary {
                true => CLabel::primary(self.file_id.clone(), it.point.as_range())
                    .with_message(it.message),
                false => CLabel::secondary(self.file_id.clone(), it.point.as_range())
                    .with_message(it.message),
            }
        }))
        .with_notes_iter(self.diagnostic.notes.into_iter().map(|it| it.into_owned()))
    }
}
