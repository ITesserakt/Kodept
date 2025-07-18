use crate::prelude::{Source, SourceFiles};
use crate::report::utils::CorrectFileId;
use crate::Execution;
use kodept_report::codespan::{CodespanSettings, Reportable};
use kodept_report::files::external::{Error, Files};
use kodept_report::message::{ReportMessage, Severity};
use kodept_report::report::Report;
use kodept_report::traits::{ad_hoc_message, IntoSpannedReportMessage, MessageBehaviour};
use std::borrow::Cow;
use std::collections::HashSet;
use std::mem::take;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::Range;
use std::sync::{Arc, Mutex};

mod utils;

pub use utils::ExtractReports;

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

#[derive(Debug)]
pub enum Reports<Impl>
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    Disabled,
    Eager {
        settings: Arc<CodespanSettings>,
        sources: Arc<Sources<Impl>>,
    },
    Lazy {
        settings: Arc<CodespanSettings>,
        sources: Arc<Sources<Impl>>,
        global_sink: Arc<Mutex<Vec<Report<()>>>>,
        local_sink: Arc<Mutex<Vec<Report>>>,
    },
}

#[derive(Debug)]
pub struct GlobalReports<Impl>(Reports<Impl>)
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>;

impl<Impl> Clone for Reports<Impl>
where
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    fn clone(&self) -> Self {
        match self {
            Reports::Disabled => Reports::Disabled,
            Reports::Eager { settings, sources } => Reports::Eager {
                settings: settings.clone(),
                sources: sources.clone(),
            },
            Reports::Lazy {
                settings,
                sources,
                global_sink,
                local_sink,
            } => Reports::Lazy {
                settings: settings.clone(),
                sources: sources.clone(),
                global_sink: global_sink.clone(),
                local_sink: local_sink.clone(),
            },
        }
    }
}

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
            settings: Arc::new(settings),
            sources: Arc::new(Sources::new()),
        })
    }

    pub fn lazy(settings: CodespanSettings) -> Self
    where
        Impl: 'static,
    {
        Self(Reports::Lazy {
            settings: Arc::new(settings),
            sources: Arc::new(Sources::new()),
            global_sink: Arc::default(),
            local_sink: Arc::default(),
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
    #[allow(private_bounds)]
    pub fn insert<F>(&self, report: Report<F>)
    where
        F: CorrectFileId,
    {
        F::insert(self, report);
    }

    fn report_failure(&self, reason: Cow<'static, str>) {
        match reason {
            Cow::Borrowed("") => {
                let message =
                    ad_hoc_message(|| ReportMessage::new(Severity::Error, "Cannot proceed"));
                <()>::insert(self, Report::from_message((), message));
            }
            otherwise => {
                let message = ad_hoc_message(|| {
                    ReportMessage::new(Severity::Error, "Cannot proceed").with_note(otherwise)
                });
                <()>::insert(self, Report::from_message((), message));
            }
        }
    }

    #[allow(private_bounds)]
    pub fn report_many<F, T>(
        &self,
        file_id: F,
        messages: impl IntoIterator<Item = T>,
    ) -> Execution<()>
    where
        F: CorrectFileId,
        T: IntoSpannedReportMessage,
    {
        let mut behaviour = None;
        for message in messages {
            match message.behaviour() {
                MessageBehaviour::FailFast { reason } => match &mut behaviour {
                    None => behaviour = Some(HashSet::from([reason])),
                    Some(reasons) => {
                        reasons.insert(reason);
                    }
                },
                MessageBehaviour::Suppress => {}
            }
            F::insert(self, Report::from_message(file_id.clone(), message));
        }
        match behaviour {
            Some(reasons) => {
                let message = ad_hoc_message(move || {
                    let mut d = ReportMessage::new(Severity::Error, "Cannot proceed");
                    for reason in reasons {
                        match reason {
                            Cow::Borrowed("") => {}
                            _ => d = d.with_note(reason),
                        }
                    }
                    d
                });
                <()>::insert(self, Report::from_message((), message));
                Break(())
            }
            None => Continue(()),
        }
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
                self.report_failure(reason);
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
                if let Some(global_sink) = Arc::get_mut(global_sink) {
                    let reports = take(global_sink.get_mut().unwrap_or_else(|it| it.into_inner()));
                    // TODO: hide implementation, so we can touch `settings` via mut reference
                    reports
                        .emit(&mut &**settings, &Global)
                        .expect("Cannot emit reports");
                }
                if let Some(local_sink) = Arc::get_mut(local_sink) {
                    let reports = take(local_sink.get_mut().unwrap_or_else(|it| it.into_inner()));
                    reports
                        .emit(&mut &**settings, &**sources)
                        .expect("Cannot emit reports");
                }
            }
        }
    }
}
