use codespan_reporting::files::Files;
use codespan_reporting::term::termcolor::{ColorSpec, StandardStream, WriteColor};
use kodept_macros::context::FileId;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::Reportable;
use std::io::Write;
use std::mem::take;
use std::sync::{Arc, LockResult, Mutex};

pub type CodespanSettings = kodept_macros::error::traits::CodespanSettings<StreamOutput>;

#[derive(Debug, Clone)]
pub enum Reports {
    Disabled,
    Eager(CodespanSettings),
    Lazy(Arc<Mutex<ReportCollector>>, CodespanSettings),
}

#[derive(Clone)]
pub enum SupportColor {
    Yes,
    No,
}

#[derive(Clone, Debug)]
pub enum StreamOutput {
    Standard(Arc<Mutex<StandardStream>>),
    NoOp,
}

const POISON_LOCK_ERROR: &str = "Lock was poisoned";

impl Reports {
    pub fn provide_collector<'a, T, F: Files<'a, FileId = FileId>>(
        &mut self,
        sources: &'a F,
        f: impl FnOnce(&mut ReportCollector) -> T,
    ) -> T {
        match self {
            Reports::Disabled => f(&mut ReportCollector::new()),
            Reports::Eager(settings) => {
                let mut collector = ReportCollector::new();
                let result = f(&mut collector);
                collector.into_collected_reports().emit(settings, sources);
                result
            }
            Reports::Lazy(collector, _) => {
                match collector.lock() {
                    Ok(mut x) => f(&mut x),
                    Err(x) => f(&mut x.into_inner())
                }
            }
        }
    }

    pub fn consume<'a, F: Files<'a, FileId = FileId>>(self, sources: &'a F) {
        match self {
            Reports::Disabled => {}
            Reports::Eager(_) => {}
            Reports::Lazy(collector, mut settings) => {
                let reports = match collector.lock() {
                    Ok(mut x) => take(&mut *x),
                    Err(x) => take(&mut *x.into_inner())
                };
                reports
                    .into_collected_reports()
                    .emit(&mut settings, sources)
            }
        }
    }
}

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
