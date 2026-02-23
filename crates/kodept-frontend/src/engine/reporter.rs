use crate::engine::reporter::sequential::Reports;
use crate::prelude::{CollectedSources, Global, SourceView};
use crate::read_code_source::SyncSource;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::resource::Resource;
use kodept_ecs::system::{Deferred, Res, SystemBuffer, SystemParam};
use kodept_report::prelude::*;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use tracing::{debug, error};

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

#[derive(Resource)]
pub struct CompilationFailed;

impl Error for CompilationFailed {}

impl Debug for CompilationFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compilation failed")
    }
}

impl Display for CompilationFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compilation failed")
    }
}

#[derive(Debug, Resource, Clone)]
pub enum Settings {
    Eager(CodespanSettings),
    Lazy(CodespanSettings),
}

#[derive(SystemParam)]
struct Config<'w, Impl: SyncSource> {
    single_source: Option<Res<'w, SourceView<Impl>>>,
    all_sources: Option<Res<'w, CollectedSources<Impl>>>,
    settings: Option<Res<'w, Settings>>,
}

#[derive(SystemParam)]
pub struct Reporter<'w, 's, Impl: SyncSource, Buffer: SystemBuffer = Reports<Impl>> {
    config: Config<'w, Impl>,
    buffer: Deferred<'s, Buffer>,
}
#[cfg(feature = "parallel")]
pub type ParallelReporter<'w, 's, Impl> = Reporter<'w, 's, Impl, parallel::ParallelReports<Impl>>;

trait Ops {
    fn mark_stop(self, stop: bool) -> Self;
    fn push_report(self, report: GenericReport);
}

impl<Impl: SyncSource> Config<'_, Impl> {
    fn report_inner(&self, message: impl IntoMessage, buffer: impl Ops) {
        let behaviour = message.behaviour();
        let buffer = match behaviour {
            MessageBehaviour::FailFast { reason } => {
                debug!("Force stopping due to fail: {reason}");
                buffer.mark_stop(true)
            }
            _ => buffer,
        };

        match (
            self.settings.as_deref(),
            self.single_source.as_ref(),
            self.all_sources.is_some(),
        ) {
            // TODO: add a way to force report to be global or not
            (Some(Settings::Eager(settings)), Some(source), _) => {
                let report = Report::from_message(*source.id, message);
                buffer.mark_stop(report.is_error());
                if let Err(e) = report.emit(&mut (settings as &_), source.all_files()) {
                    error!("Cannot emit report: {e}");
                }
            }
            (Some(Settings::Eager(settings)), None, _) => {
                let report = Report::from_message((), message);
                buffer.mark_stop(report.is_error());
                if let Err(e) = report.emit(&mut (settings as &_), &Global) {
                    error!("Cannot emit report: {e}");
                }
            }
            (Some(Settings::Lazy(_)), Some(source), _) => {
                buffer.push_report(GenericReport::Single(Report::from_message(
                    *source.id, message,
                )));
            }
            (Some(Settings::Lazy(_)), None, _) => {
                buffer.push_report(GenericReport::Global(Report::from_message((), message)));
            }
            (None, _, _) => {
                // TODO: know whether report is erroneous slightly ahead
                let report = Report::from_message((), message);
                buffer.mark_stop(report.is_error());
            }
        }
    }
}

struct Helper<F, T>(F, PhantomData<fn() -> T>);
impl<F, T> IntoMessage for Helper<F, T>
where
    T: Message,
    F: FnOnce() -> T,
{
    type Message = T;

    fn into_message(self) -> Self::Message {
        let func = self.0;
        func()
    }
}

mod sequential {
    use crate::engine::reporter::{
        CompilationFailed, GenericReport, Helper, Ops, Reporter, Settings,
    };
    use crate::read_code_source::SyncSource;
    use crate::report::Global;
    use crate::source_files::{CollectedSources, SourceView};
    use kodept_ecs::system::{SystemBuffer, SystemMeta};
    use kodept_ecs::world::World;
    use kodept_report::codespan::Reportable;
    use kodept_report::traits::{IntoMessage, Message};
    use std::marker::PhantomData;
    use tracing::error;

    pub struct Reports<Impl> {
        pub(super) deferred_reports: Vec<GenericReport>,
        pub(super) should_stop: bool,
        pub(super) _phantom: PhantomData<fn() -> Impl>,
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

    impl<Impl: SyncSource> SystemBuffer for Reports<Impl> {
        fn apply(&mut self, _: &SystemMeta, world: &mut World) {
            let mut any_error = false;
            world.try_resource_scope(|world, mut settings| {
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

                match &mut *settings {
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
            });
            if any_error || self.should_stop {
                self.should_stop = false;
                world.insert_resource(CompilationFailed);
            }
        }
    }

    impl<Impl> Ops for &mut Reports<Impl> {
        fn mark_stop(self, stop: bool) -> Self {
            self.should_stop = stop;
            self
        }

        fn push_report(self, report: GenericReport) {
            self.deferred_reports.push(report);
        }
    }

    impl<Impl: SyncSource> Reporter<'_, '_, Impl> {
        #[inline]
        pub fn report(&mut self, message: impl IntoMessage) {
            self.config.report_inner(message, &mut *self.buffer);
        }

        #[inline]
        pub fn report_ad_hoc<T>(&mut self, message: impl FnOnce() -> T)
        where
            T: Message,
        {
            self.report(Helper(message, PhantomData))
        }
    }
}

#[cfg(feature = "parallel")]
mod parallel {
    use crate::engine::reporter::{GenericReport, Helper, Ops, Reporter, Reports};
    use crate::read_code_source::SyncSource;
    use kodept_ecs::system::{SystemBuffer, SystemMeta};
    use kodept_ecs::utils::Parallel;
    use kodept_ecs::world::World;
    use kodept_report::traits::{IntoMessage, Message};
    use std::marker::PhantomData;
    use std::sync::atomic::{AtomicBool, Ordering};

    pub struct ParallelReports<Impl> {
        sinks: Parallel<Vec<GenericReport>>,
        should_stop: AtomicBool,
        _phantom: PhantomData<fn() -> Impl>,
    }

    impl<Impl> Default for ParallelReports<Impl> {
        fn default() -> Self {
            Self {
                sinks: Parallel::default(),
                should_stop: AtomicBool::new(false),
                _phantom: PhantomData,
            }
        }
    }

    impl<Impl: SyncSource> SystemBuffer for ParallelReports<Impl> {
        fn apply(&mut self, system_meta: &SystemMeta, world: &mut World) {
            let mut local_reports = Reports {
                _phantom: PhantomData::<fn() -> Impl>,
                should_stop: *self.should_stop.get_mut(),
                deferred_reports: vec![],
            };
            self.sinks.drain_into(&mut local_reports.deferred_reports);
            local_reports.apply(system_meta, world);
            *self.should_stop.get_mut() = local_reports.should_stop;
        }
    }

    impl<Impl> Ops for &ParallelReports<Impl> {
        fn mark_stop(self, stop: bool) -> Self {
            self.should_stop.store(stop, Ordering::Relaxed);
            self
        }

        fn push_report(self, report: GenericReport) {
            self.sinks.scope(|it| it.push(report));
        }
    }

    impl<Impl: SyncSource> Reporter<'_, '_, Impl, ParallelReports<Impl>> {
        pub fn report(&self, message: impl IntoMessage) {
            self.config.report_inner(message, &*self.buffer);
        }

        pub fn report_ad_hoc<T: Message>(&self, f: impl FnOnce() -> T) {
            self.report(Helper(f, PhantomData))
        }
    }
}
