use crate::traits::{IntoSpannedReportMessage, SpannedReportMessage};
use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Report<FileId = crate::FileId> {
    diagnostic: Diagnostic<FileId>,
}

impl<FileId> Report<FileId> {
    fn from_raw_message_with_code<T>(
        file_id: FileId,
        message: T,
        code: String,
    ) -> Diagnostic<FileId>
    where
        T: SpannedReportMessage,
        FileId: Clone,
    {
        let raw_message = message.into();

        let mut diagnostic = Diagnostic::new(raw_message.severity.into_codespan());
        diagnostic.message = raw_message.message.to_string();
        diagnostic.code = Some(code);
        diagnostic.labels = raw_message
            .labels
            .into_iter()
            .map(move |it| {
                if it.primary {
                    Label::primary(file_id.clone(), it.point.as_range()).with_message(it.message)
                } else {
                    Label::secondary(file_id.clone(), it.point.as_range()).with_message(it.message)
                }
            })
            .collect();
        diagnostic.notes = raw_message
            .notes
            .into_iter()
            .map(|it| it.to_string())
            .collect();
        diagnostic
    }

    #[must_use]
    pub fn from_message<T>(file_id: FileId, msg: T) -> Self
    where
        T: IntoSpannedReportMessage,
        FileId: Clone,
    {
        let code = format!("{:0>8X}", msg.code());
        let diagnostic = Self::from_raw_message_with_code(file_id, msg.into_message(), code);
        Self { diagnostic }
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.diagnostic.severity, Severity::Error)
    }

    pub fn into_inner(self) -> Diagnostic<FileId> {
        self.diagnostic
    }
}
