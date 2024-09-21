use codespan_reporting::files::Files;
use codespan_reporting::term::termcolor::{ColorSpec, StandardStream, WriteColor};
use kodept_macros::context::FileId;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::Reportable;
use std::io::Write;
use std::mem::take;
use std::sync::{Arc, Mutex};

pub trait ProvideCollector<Id> {
    fn provide_collector<'a, T, F>(
        &mut self,
        sources: &'a F,
        f: impl FnOnce(&mut ReportCollector<F::FileId>) -> T,
    ) -> T
    where
        F: Files<'a, FileId = Id>;
}

pub trait ConsumeCollector<'a, Id> {
    fn consume<F>(self, sources: &'a F)
    where
        F: Files<'a, FileId = Id>;
}

pub type CodespanSettings = kodept_macros::error::traits::CodespanSettings<StreamOutput>;

#[derive(Debug, Clone)]
pub enum Reports {
    Disabled,
    Eager(CodespanSettings),
    Lazy {
        local_reports: Arc<Mutex<ReportCollector>>,
        global_reports: Arc<Mutex<ReportCollector<()>>>,
        settings: CodespanSettings,
    },
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

impl<'a> ConsumeCollector<'a, FileId> for Reports {
    fn consume<F>(self, sources: &'a F)
    where
        F: Files<'a, FileId = FileId>,
    {
        match self {
            Reports::Disabled => {}
            Reports::Eager(_) => {}
            Reports::Lazy {
                local_reports,
                mut settings,
                ..
            } => {
                let mut lock = local_reports.lock().unwrap_or_else(|e| e.into_inner());
                let collector = take(&mut *lock);
                collector
                    .into_collected_reports()
                    .emit(&mut settings, sources)
            }
        }
    }
}

impl ConsumeCollector<'static, ()> for Reports {
    fn consume<F>(self, sources: &'static F)
    where
        F: Files<'static, FileId = ()>,
    {
        match self {
            Reports::Disabled => {}
            Reports::Eager(_) => {}
            Reports::Lazy {
                global_reports,
                mut settings,
                ..
            } => {
                let mut lock = global_reports.lock().unwrap_or_else(|e| e.into_inner());
                let collector = take(&mut *lock);
                collector
                    .into_collected_reports()
                    .emit(&mut settings, sources)
            }
        }
    }
}

impl ProvideCollector<FileId> for Reports {
    fn provide_collector<'a, T, F>(
        &mut self,
        sources: &'a F,
        f: impl FnOnce(&mut ReportCollector<F::FileId>) -> T,
    ) -> T
    where
        F: Files<'a, FileId = FileId>,
    {
        match self {
            Reports::Disabled => f(&mut ReportCollector::new()),
            Reports::Eager(settings) => {
                let mut collector = ReportCollector::new();
                let result = f(&mut collector);
                collector.into_collected_reports().emit(settings, sources);
                result
            }
            Reports::Lazy { local_reports, .. } => {
                let mut lock = local_reports.lock().unwrap_or_else(|e| e.into_inner());
                f(&mut *lock)
            }
        }
    }
}

impl ProvideCollector<()> for Reports {
    fn provide_collector<'a, T, F>(
        &mut self,
        sources: &'a F,
        f: impl FnOnce(&mut ReportCollector<F::FileId>) -> T,
    ) -> T
    where
        F: Files<'a, FileId = ()>,
    {
        match self {
            Reports::Disabled => f(&mut ReportCollector::new()),
            Reports::Eager(settings) => {
                let mut collector = ReportCollector::new();
                let result = f(&mut collector);
                collector.into_collected_reports().emit(settings, sources);
                result
            }
            Reports::Lazy { global_reports, .. } => {
                let mut lock = global_reports.lock().unwrap_or_else(|e| e.into_inner());
                f(&mut *lock)
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
