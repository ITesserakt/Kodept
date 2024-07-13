use crate::error::report::{Report, ReportMessage};
use crate::error::ErrorReported;
use codespan_reporting::diagnostic::{Diagnostic, Severity};
use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::WriteColor;
use codespan_reporting::term::Config;
use extend::ext;
use kodept_core::file_relative::{CodePath, FileRelative};

#[derive(Clone)]
pub struct CodespanSettings<S> {
    pub config: Config,
    pub stream: S,
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

impl Reportable for Report {
    type FileId = ();

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        self.into_diagnostic().emit(settings, source)
    }
}

impl<E: std::error::Error> Reportable for FileRelative<&E> {
    type FileId = ();

    fn emit<'f, W: WriteColor, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        let report = Report::new(
            &self.filepath,
            vec![],
            ReportMessage::new(Severity::Error, "external", self.value.to_string()),
        );
        report.emit(settings, source)
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
    fn or_emit<'f, W: WriteColor, F: Files<'f, FileId = ()>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
        filepath: CodePath,
    ) -> Result<T, ErrorReported> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                let file_relative = FileRelative {
                    value: &e,
                    filepath,
                };
                file_relative
                    .emit(settings, source)
                    .expect("Cannot emit diagnostics");
                Err(ErrorReported::with_cause(e))
            }
        }
    }
}
