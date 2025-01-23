use crate::prelude::{Source, SourceFiles};
use crate::report::utils::CorrectFileId;
use crate::Execution;
use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::StandardStream;
use kodept_report::error::report::{
    ad_hoc_message, IntoSpannedReportMessage, MessageBehaviour, Report, ReportMessage, Severity,
};
use kodept_report::error::traits::Reportable;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::Range;
use std::sync::{Arc, Mutex};

mod utils;

type CodespanSettings = kodept_report::error::traits::CodespanSettings<StandardStream>;
type Sources<Impl> = SourceFiles<Impl>;

pub struct Global;

impl<'a> Files<'a> for Global {
    type FileId = ();
    type Name = &'static str;
    type Source = &'static str;

    fn name(&'a self, _: Self::FileId) -> Result<Self::Name, Error> {
        Ok("<global level>")
    }

    fn source(&'a self, _: Self::FileId) -> Result<Self::Source, Error> {
        Err(Error::FileMissing)
    }

    fn line_index(&'a self, _: Self::FileId, _: usize) -> Result<usize, Error> {
        Err(Error::FileMissing)
    }

    fn line_range(&'a self, _: Self::FileId, _: usize) -> Result<Range<usize>, Error> {
        Err(Error::FileMissing)
    }
}

pub enum Reports<Impl>
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    Disabled,
    Eager {
        settings: CodespanSettings,
        sources: Arc<Sources<Impl>>,
    },
    Lazy {
        settings: CodespanSettings,
        sources: Arc<Sources<Impl>>,
        global_sink: Mutex<Vec<Report<()>>>,
        local_sink: Mutex<Vec<Report>>,
    },
}

pub struct GlobalReports<Impl>(Reports<Impl>)
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>;

impl<Impl> GlobalReports<Impl>
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    pub fn disabled() -> Self {
        Self(Reports::Disabled)
    }

    pub fn eager(settings: CodespanSettings) -> Self
    where
        Impl: 'static,
    {
        Self(Reports::Eager {
            settings,
            sources: Arc::new(Sources::new()),
        })
    }

    pub fn lazy(settings: CodespanSettings) -> Self
    where
        Impl: 'static,
    {
        Self(Reports::Lazy {
            settings,
            sources: Arc::new(Sources::new()),
            global_sink: Mutex::new(vec![]),
            local_sink: Mutex::new(vec![]),
        })
    }

    pub fn report<T>(&self, message: T) -> Execution<()>
    where
        T: IntoSpannedReportMessage,
    {
        self.0.report((), message)
    }

    pub fn upgrade(mut self, updated: Arc<Sources<Impl>>) -> Reports<Impl> {
        match &mut self.0 {
            Reports::Disabled => {}
            Reports::Eager { sources, .. } => *sources = updated,
            Reports::Lazy { sources, .. } => *sources = updated,
        };
        self.0
    }
}

impl<Impl> Reports<Impl>
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    pub fn insert<F>(&self, report: Report<F>)
    where 
        F: CorrectFileId {
        F::insert(self, report);
    }
    
    #[allow(private_bounds)]
    pub fn report<F, T>(&self, file_id: F, message: T) -> Execution<()>
    where
        T: IntoSpannedReportMessage,
        F: CorrectFileId,
    {
        let behaviour = message.behaviour();
        F::insert(self, Report::from_message(file_id, message));
        match behaviour {
            MessageBehaviour::FailFast { reason } => {
                let message = ad_hoc_message(|| {
                    ReportMessage::new(Severity::Error, "Cannot proceed".to_string())
                        .with_note(reason)
                });
                <()>::insert(self, Report::from_message((), message));
                Break(())
            }
            MessageBehaviour::Suppress => Continue(()),
        }
    }
}

impl<Impl> Drop for Reports<Impl>
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    fn drop(&mut self) {
        match self {
            Reports::Disabled => {}
            Reports::Eager { .. } => {}
            Reports::Lazy {
                settings,
                sources,
                global_sink,
                local_sink,
            } => {
                let global_sink = global_sink.get_mut().unwrap_or_else(|it| it.into_inner());
                let local_sink = local_sink.get_mut().unwrap_or_else(|it| it.into_inner());

                global_sink
                    .drain(..)
                    .for_each(|it| it.emit(settings, &Global));

                local_sink
                    .drain(..)
                    .for_each(|it| it.emit(settings, &**sources));
            }
        }
    }
}
