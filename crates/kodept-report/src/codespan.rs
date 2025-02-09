use crate::report::Report;
use codespan_reporting::term::termcolor::{StandardStream, WriteColor};
use codespan_reporting::term::{ColorArg, Config};
use derive_more::Constructor;
use crate::files::external::{Files, Error};

pub mod external {
    pub use codespan_reporting::term::{Config, ColorArg, DisplayStyle};
}

#[derive(Clone, Debug, Constructor)]
pub struct CodespanSettings<S = StandardStream> {
    config: Config,
    stream: S,
}

pub trait Settings {
    fn unpack(&mut self) -> (&Config, impl WriteColor);
}

pub trait Reportable {
    type FileId;

    fn emit<'f, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut impl Settings,
        source: &'f F,
    ) -> Result<(), Error>;
}

impl CodespanSettings {
    pub fn stderr(config: Config, color: ColorArg) -> Self {
        Self::new(config, StandardStream::stderr(color.0))
    }
} 

impl<FileId> Reportable for Report<FileId> {
    type FileId = FileId;

    fn emit<'f, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut impl Settings,
        source: &'f F,
    ) -> Result<(), Error> {
        let (config, mut writer) = settings.unpack();
        codespan_reporting::term::emit(&mut writer, config, source, &self.into_inner())
    }
}

impl<R> Reportable for Vec<R>
where
    R: Reportable,
{
    type FileId = R::FileId;

    fn emit<'f, F: Files<'f, FileId = Self::FileId>>(
        self,
        settings: &mut impl Settings,
        source: &'f F,
    ) -> Result<(), Error> {
        for item in self {
            item.emit(settings, source)?;
        }
        Ok(())
    }
}

impl Settings for &CodespanSettings<StandardStream> {
    fn unpack(&mut self) -> (&Config, impl WriteColor) {
        let lock = self.stream.lock();
        (&self.config, lock)
    }
}

impl<S> Settings for &mut CodespanSettings<S>
where
    S: WriteColor,
{
    fn unpack(&mut self) -> (&Config, impl WriteColor) {
        (&self.config, &mut self.stream)
    }
}
