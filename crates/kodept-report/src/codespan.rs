use std::io::Write;
use std::sync::Arc;
use crate::files::external::{Error, Files};
use crate::report::Report;
use codespan_reporting::term::termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use codespan_reporting::term::Config;

pub mod external {
    pub use codespan_reporting::term::{termcolor::ColorChoice, Config, DisplayStyle};
}

#[derive(Debug, Clone)]
pub struct ClonableStandardStream(Arc<StandardStream>);

#[derive(Clone, Debug)]
pub struct CodespanSettings<S = ClonableStandardStream> {
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

impl<S> CodespanSettings<S> {
    pub fn new(config: Config, stream: S) -> Self {
        Self { config, stream }
    }
}

impl CodespanSettings {
    pub fn stderr(config: Config, color: ColorChoice) -> Self {
        Self::new(config, ClonableStandardStream(Arc::new(StandardStream::stderr(color))))
    }
}

impl Write for ClonableStandardStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // fast path
        if let Some(stream) = Arc::get_mut(&mut self.0) {
            stream.write(buf)
        } else {
            let mut lock = self.0.lock();
            lock.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(stream) = Arc::get_mut(&mut self.0) {
            stream.flush()
        } else {
            let mut lock = self.0.lock();
            lock.flush()
        }
    }
}

impl WriteColor for ClonableStandardStream {
    fn supports_color(&self) -> bool {
        self.0.supports_color()
    }

    fn set_color(&mut self, spec: &ColorSpec) -> std::io::Result<()> {
        if let Some(stream) = Arc::get_mut(&mut self.0) {
            stream.set_color(spec)
        } else {
            let mut lock = self.0.lock();
            lock.set_color(spec)
        }
    }

    fn reset(&mut self) -> std::io::Result<()> {
        if let Some(stream) = Arc::get_mut(&mut self.0) {
            stream.reset()
        } else {
            let mut lock = self.0.lock();
            lock.reset()
        }
    }
}

impl<FileId> Reportable for Report<FileId>
where
    FileId: Clone,
{
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

impl Settings for &CodespanSettings<ClonableStandardStream> {
    fn unpack(&mut self) -> (&Config, impl WriteColor) {
        (&self.config, self.stream.clone())
    }
}

impl<S> Settings for CodespanSettings<S>
where
    S: WriteColor,
{
    fn unpack(&mut self) -> (&Config, impl WriteColor) {
        (&self.config, &mut self.stream)
    }
}
