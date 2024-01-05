use std::io::Write;
use std::sync::{Arc, Mutex};

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::Config;
use codespan_reporting::term::termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use kodept_macros::error::report::Report;

#[derive(Clone)]
pub struct CodespanSettings<S = StreamOutput> {
    pub config: Config,
    pub stream: S,
}

pub trait ReportExt: Sized {
    fn emit<'f, W: WriteColor, F: Files<'f, FileId = ()>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error>;
}

#[derive(Clone)]
pub enum SupportColor {
    Yes,
    No,
}

#[derive(Clone)]
pub enum StreamOutput {
    Standard(Arc<Mutex<StandardStream>>),
    NoOp,
}

impl Default for CodespanSettings<StandardStream> {
    fn default() -> Self {
        Self {
            config: Default::default(),
            stream: StandardStream::stderr(ColorChoice::Auto),
        }
    }
}

impl ReportExt for Report {
    fn emit<'f, W: WriteColor, F: Files<'f, FileId = ()>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        codespan_reporting::term::emit(
            &mut settings.stream,
            &settings.config,
            source,
            &self.into_diagnostic(),
        )
    }
}

impl ReportExt for Diagnostic<()> {
    fn emit<'f, W: WriteColor, F: Files<'f, FileId = ()>>(
        self,
        settings: &mut CodespanSettings<W>,
        source: &'f F,
    ) -> Result<(), Error> {
        codespan_reporting::term::emit(&mut settings.stream, &settings.config, source, &self)
    }
}

const POISON_LOCK_ERROR: &str = "Lock was poisoned";

impl Write for StreamOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).write(buf),
            StreamOutput::NoOp => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).flush(),
            StreamOutput::NoOp => Ok(()),
        }
    }
}

impl WriteColor for StreamOutput {
    fn supports_color(&self) -> bool {
        match self {
            StreamOutput::Standard(x) => {
                let guard = x.lock();
                match guard {
                    Ok(it) => it.supports_color(),
                    Err(it) => it.get_ref().supports_color(),
                }
            }
            StreamOutput::NoOp => false,
        }
    }

    fn set_color(&mut self, spec: &ColorSpec) -> std::io::Result<()> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).set_color(spec),
            StreamOutput::NoOp => Ok(()),
        }
    }

    fn reset(&mut self) -> std::io::Result<()> {
        match self {
            StreamOutput::Standard(x) => x.lock().expect(POISON_LOCK_ERROR).reset(),
            StreamOutput::NoOp => Ok(()),
        }
    }
}
