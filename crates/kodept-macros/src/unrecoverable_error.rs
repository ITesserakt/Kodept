use crate::error::report::Report;
use crate::error::traits::{CodespanSettings, Reportable};
use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::WriteColor;
use derive_more::{From, Unwrap};

#[derive(Debug, From, Unwrap)]
pub enum UnrecoverableError {
    Report(Report),
}

impl UnrecoverableError {
    pub fn into_report(self) -> Report {
        match self {
            UnrecoverableError::Report(x) => x,
        }
    }
}

impl Reportable for UnrecoverableError {
    type FileId = ();

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        self.into_report().emit(settings, source)
    }
}
