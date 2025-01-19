use crate::source_files::Sources;
use bevy_ecs::prelude::{Event, EventWriter, Events, In, IntoSystem, Res, ResMut, Trigger};
use codespan_reporting::term::termcolor::{ColorSpec, StandardStream, WriteColor};
use kodept_frontend::external::Resource;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::{ExitEvent, Plugin};
use kodept_frontend::prelude::{GlobalReports};
use kodept_report::error::report::{IntoSpannedReportMessage, MessageBehaviour, Report};
use kodept_report::error::traits::Reportable;
use kodept_report::FileId;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub type CodespanSettings = kodept_report::error::traits::CodespanSettings<StreamOutput>;

#[derive(Debug, Event)]
pub struct ReportEmitted {
    report: Report,
    _behaviour: MessageBehaviour,
}

#[derive(Debug, Event)]
pub struct GlobalReportEmitted {
    report: Report<()>,
    _behaviour: MessageBehaviour,
}

#[derive(Debug, Resource)]
pub enum Reports {
    Disabled,
    Eager(CodespanSettings),
    Lazy(CodespanSettings),
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

pub struct ReportsPlugin;

const POISON_LOCK_ERROR: &str = "Lock was poisoned";

impl ReportEmitted {
    pub fn new(file_id: FileId, message: impl IntoSpannedReportMessage) -> Self {
        let behaviour = message.behaviour();
        Self {
            report: Report::from_message(file_id, message),
            _behaviour: behaviour,
        }
    }
}

impl GlobalReportEmitted {
    pub fn new(message: impl IntoSpannedReportMessage) -> Self {
        let behaviour = message.behaviour();
        Self {
            report: Report::from_message((), message),
            _behaviour: behaviour,
        }
    }
}

impl Plugin for ReportsPlugin {
    fn build(self, world: &mut Frontend) {
        world
            .add_event::<GlobalReportEmitted>()
            .add_event::<ReportEmitted>();
        // Draining systems that applied at application exit
        world.add_observer((|_: Trigger<ExitEvent>| true).pipe(Reports::drain));
        world.add_observer((|_: Trigger<ExitEvent>| true).pipe(Reports::global_drain));
        
        // Draining systems that applied every tick
        world.add_freestanding_systems((|| false).pipe(Reports::drain));
        world.add_freestanding_systems((|| false).pipe(Reports::global_drain));
    }
}

impl Reports {
    pub fn stop_and_report<E: Error + 'static>(
        In(result): In<Result<(), E>>,
        mut writer: EventWriter<GlobalReportEmitted>,
    ) {
        let Err(error) = result else {
            return;
        };
        writer.send(GlobalReportEmitted::new(error));
    }

    pub fn pipe_or_report<T, E: Error + 'static>(
        In(result): In<Result<T, E>>,
        mut writer: EventWriter<GlobalReportEmitted>,
    ) -> Option<T> {
        match result {
            Ok(x) => Some(x),
            Err(e) => {
                writer.send(GlobalReportEmitted::new(e));
                None
            }
        }
    }

    fn drain(
        In(last): In<bool>,
        mut events: ResMut<Events<ReportEmitted>>,
        mut reports: ResMut<Reports>,
        sources: Option<Res<Sources>>,
    ) {
        let Some(sources) = sources else {
            return;
        };
        match &mut *reports {
            Reports::Disabled => {
                _ = events.drain();
            }
            Reports::Eager(settings) => {
                events
                    .drain()
                    .map(|it| it.report)
                    .for_each(|it| it.emit(settings, &**sources));
            }
            Reports::Lazy(settings) if last => {
                events
                    .drain()
                    .map(|it| it.report)
                    .for_each(|it| it.emit(settings, &**sources));
            }
            Reports::Lazy(_) => {}
        }
    }

    fn global_drain(
        In(last): In<bool>,
        mut events: ResMut<Events<GlobalReportEmitted>>,
        mut reports: ResMut<Reports>,
    ) {
        match &mut *reports {
            Reports::Disabled => {
                _ = events.drain();
            }
            Reports::Eager(settings) => {
                events
                    .drain()
                    .map(|it| it.report)
                    .for_each(|it| it.emit(settings, &GlobalReports));
            }
            Reports::Lazy(settings) if last => {
                events
                    .drain()
                    .map(|it| it.report)
                    .for_each(|it| it.emit(settings, &GlobalReports));
            }
            Reports::Lazy(_) => {}
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
