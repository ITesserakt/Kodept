use crate::prelude::{CollectedSources, Global, Source, SourceView};
use bevy_ecs::prelude::*;
use bevy_ecs::system::{SystemBuffer, SystemMeta, SystemParam};
use kodept_report::prelude::*;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use tracing::{debug, error, trace};

#[derive(Debug)]
enum GenericReport {
    Single(Report),
    Global(Report<()>),
}

impl GenericReport {
    #[inline]
    fn is_error(&self) -> bool {
        match self {
            GenericReport::Single(x) => x.is_error(),
            GenericReport::Global(x) => x.is_error(),
        }
    }
}

struct Reports<Impl> {
    deferred_reports: Vec<GenericReport>,
    should_stop: bool,
    _phantom: PhantomData<fn() -> Impl>,
}

impl<Impl> Default for Reports<Impl> {
    #[inline]
    fn default() -> Self {
        Self {
            deferred_reports: vec![],
            should_stop: false,
            _phantom: PhantomData,
        }
    }
}

impl<Impl> SystemBuffer for Reports<Impl>
where
    Impl: Send + Sync + 'static,
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    fn apply(&mut self, _: &SystemMeta, world: &mut World) {
        world.resource_scope(|world, mut settings| {
            let all_files = {
                world
                    .get_resource::<CollectedSources<Impl>>()
                    .map(|it| it.inner.as_ref())
                    .or_else(|| {
                        world
                            .get_resource::<SourceView<Impl>>()
                            .map(|it| it.all_files())
                    })
            };
            let mut any_error = false;

            match &mut *settings {
                Settings::Disabled => {
                    self.deferred_reports.clear();
                }
                Settings::Eager(_) if !self.deferred_reports.is_empty() => {
                    unreachable!("All reports should have been reported already")
                }
                Settings::Eager(_) => {}
                Settings::Lazy(settings) => {
                    for report in self.deferred_reports.drain(..) {
                        any_error |= report.is_error();
                        let emit_result = match report {
                            GenericReport::Single(x) => x.emit(
                                settings,
                                all_files.expect(
                                    "Resource `CollectedSources` or `SourceView` are not found",
                                ),
                            ),
                            GenericReport::Global(x) => x.emit(settings, &Global),
                        };
                        if let Err(error) = emit_result {
                            error!("Cannot emit reports: {error}");
                        }
                    }
                }
            }
            if any_error || self.should_stop {
                self.should_stop = false;
                StopEngine::stop()
            }
        })
    }
}

#[derive(Debug)]
pub struct StopEngine;

impl StopEngine {
    fn stop() -> ! {
        std::panic::resume_unwind(Box::new(StopEngine))
    }
}

impl Error for StopEngine {}

impl Display for StopEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compilation failed")
    }
}

#[derive(Debug, Resource, Default, Clone)]
pub enum Settings {
    #[default]
    Disabled,
    Eager(CodespanSettings),
    Lazy(CodespanSettings),
}

#[derive(SystemParam)]
pub struct Reporter<'w, 's, Impl>
where
    Impl: Send + Sync + 'static,
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    single_source: Option<Res<'w, SourceView<Impl>>>,
    all_sources: Option<Res<'w, CollectedSources<Impl>>>,
    settings: Res<'w, Settings>,
    buffer: Deferred<'s, Reports<Impl>>,
}

impl<Impl> Reporter<'_, '_, Impl>
where
    Impl: Send + Sync + 'static,
    Impl: for<'a> Source<Ref<'a>: AsRef<str>>,
{
    pub fn report(&mut self, message: impl IntoSpannedReportMessage) {
        let behaviour = message.behaviour();
        trace!(?behaviour, "Reported new message");

        match (
            self.settings.as_ref(),
            self.single_source.as_ref(),
            self.all_sources.is_some(),
        ) {
            // TODO: add a way to force report to be global or not
            (Settings::Eager(settings), Some(source), _) => {
                let report = Report::from_message(*source.id, message);
                if let Err(e) = report.emit(&mut (settings as &_), source.all_files()) {
                    error!("Cannot emit report: {e}");
                }
            }
            (Settings::Eager(settings), None, _) => {
                let report = Report::from_message((), message);
                if let Err(e) = report.emit(&mut (settings as &_), &Global) {
                    error!("Cannot emit report: {e}");
                }
            }
            (Settings::Lazy(_), Some(source), _) => {
                self.buffer
                    .deferred_reports
                    .push(GenericReport::Single(Report::from_message(
                        *source.id, message,
                    )));
            }
            (Settings::Lazy(_), None, _) => {
                self.buffer
                    .deferred_reports
                    .push(GenericReport::Global(Report::from_message((), message)));
            }
            (Settings::Disabled, _, _) => {}
        }
        if let MessageBehaviour::FailFast { reason } = behaviour {
            debug!("Force stopping due to fail: {reason}");
            self.buffer.should_stop = true;
        }
    }

    #[inline]
    pub fn report_ad_hoc<T>(&mut self, message: impl FnOnce() -> T)
    where
        T: SpannedReportMessage,
    {
        struct Helper<F, T>(F, PhantomData<fn() -> T>);
        impl<F, T> IntoSpannedReportMessage for Helper<F, T>
        where
            T: SpannedReportMessage,
            F: FnOnce() -> T,
        {
            type Message = T;

            fn into_message(self) -> Self::Message {
                let func = self.0;
                func()
            }
        }

        self.report(Helper(message, PhantomData))
    }
}
