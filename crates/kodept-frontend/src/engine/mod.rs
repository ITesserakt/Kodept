use crate::engine::function_impls::InlineFunctionPhase;
use crate::engine::reporter::CompilationFailed;
use crate::prelude::Global;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy_ecs::system::{IntoObserverSystem, ScheduleSystem};
use kodept_report::codespan::external::{ColorChoice, Config, DisplayStyle};
use kodept_report::message::Severity;
use kodept_report::prelude::{CodespanSettings, Diagnostic, Report, Reportable, ad_hoc_message};
use std::any::Any;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::panic::AssertUnwindSafe;
use std::sync::OnceLock;

pub mod macros;
pub mod reporter;
pub mod utils;

#[derive(Debug)]
enum Location {
    Unknown,
    Known {
        filename: String,
        column: u32,
        line: u32,
    },
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Unknown => write!(f, "<unknown>:1:1"),
            Location::Known {
                filename,
                column,
                line,
            } => write!(f, "{filename}:{line}:{column}"),
        }
    }
}

#[derive(ScheduleLabel, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Startup;

#[derive(Debug)]
pub struct Engine {
    engine_world: World,
}

#[derive(Debug, Component)]
pub struct SubEngine {
    inner: Engine,
}

pub trait Phase: Sized {
    type Set: IntoSystemSet<()> + Default;

    fn build(self, engine: &mut PhaseEngine<Self>);
}

pub trait Plugin {
    fn build(self, engine: &mut Engine);
}

mod function_impls {
    use crate::engine::{Engine, Phase, PhaseEngine, Plugin};
    use bevy_ecs::prelude::SystemSet;

    #[derive(Debug, SystemSet, Copy, Clone, PartialEq, Hash, Default, Eq)]
    pub struct SingletonSet;

    pub enum InlineFunctionPhase {}

    impl Phase for InlineFunctionPhase {
        type Set = SingletonSet;

        fn build(self, _: &mut PhaseEngine<Self>) {
            match self {}
        }
    }

    impl<F> Plugin for F
    where
        F: FnOnce(&mut Engine),
    {
        fn build(self, engine: &mut Engine) {
            self(engine)
        }
    }
}

pub struct Chaining<'a, P> {
    engine: &'a mut Engine,
    _phantom: PhantomData<fn() -> P>,
}

impl<P> Chaining<'_, P> {
    pub fn install<Q>(&mut self, phase: Q) -> Chaining<'_, Q>
    where
        P: Phase,
        Q: Phase,
    {
        self.engine.with_schedule(Startup, |schedule| {
            let label1 = P::Set::default();
            let label2 = Q::Set::default();
            schedule.configure_sets((label1.into_system_set(), label2.into_system_set()).chain());
        });
        self.engine.install(phase)
    }
}

pub struct PhaseEngine<'a, P> {
    engine: &'a mut Engine,
    pub instrumented: bool,
    _phantom: PhantomData<fn() -> P>,
}

impl<P> PhaseEngine<'_, P>
where
    P: Phase,
{
    pub fn add_systems<M>(&mut self, config: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        let label = P::Set::default();
        let config = if self.instrumented {
            let name = std::any::type_name::<P>();
            utils::instrument::instrument(config, name)
        } else {
            config.into_configs()
        };
        self.engine
            .add_systems(Startup, config.in_set(label.into_system_set()))
    }
}

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedules = Schedules::new();
        let startup = schedules.entry(Startup);

        // execution deadlocks if `ExecutorKind` is MultiThreaded...
        startup.set_executor_kind(ExecutorKind::SingleThreaded);

        world.insert_resource(schedules);

        Self {
            engine_world: world,
        }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_plugin_if(&mut self, condition: bool, plugin: impl Plugin) -> &mut Self {
        if condition {
            self.add_plugin(plugin);
        }
        self
    }

    pub fn install<P: Phase>(&mut self, phase: P) -> Chaining<'_, P> {
        phase.build(&mut PhaseEngine {
            engine: self,
            instrumented: true,
            _phantom: PhantomData,
        });
        Chaining {
            engine: self,
            _phantom: PhantomData,
        }
    }

    pub fn install_inline_phase(
        &mut self,
        build: impl FnOnce(&mut PhaseEngine<InlineFunctionPhase>),
    ) {
        build(&mut PhaseEngine {
            engine: self,
            instrumented: true,
            _phantom: PhantomData,
        });
    }

    pub fn run(&mut self) -> Result<(), CompilationFailed> {
        static PANIC_LOCATION: OnceLock<Location> = OnceLock::new();
        static PANIC_BACKTRACE: OnceLock<Backtrace> = OnceLock::new();
        std::panic::set_hook(Box::new(|info| {
            let location = info.location();
            _ = PANIC_LOCATION.set(location.map_or(Location::Unknown, |it| Location::Known {
                filename: it.file().to_string(),
                column: it.column(),
                line: it.line(),
            }));
            _ = PANIC_BACKTRACE.set(Backtrace::capture());
        }));

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.engine_world.run_schedule(Startup);
        }));
        if let Err(error) = result {
            Self::report_panic(
                error,
                PANIC_LOCATION.get_or_init(|| Location::Unknown),
                PANIC_BACKTRACE.get_or_init(|| Backtrace::disabled()),
            );
            return Err(CompilationFailed);
        }
        Ok(())
    }

    #[inline]
    fn report_panic(error: Box<dyn Any + Send>, location: &Location, backtrace: &Backtrace) {
        let payload = if let Some(s) = error.downcast_ref::<&str>() {
            Some(s.to_string())
        } else if let Some(s) = error.downcast_ref::<String>() {
            Some(s.to_string())
        } else if let Some(_) = error.downcast_ref::<CompilationFailed>() {
            return;
        } else {
            None
        };

        let report = Report::from_message(
            (),
            ad_hoc_message(|| {
                let mut diagnostic = Diagnostic::new(Severity::Bug)
                    .with_message("Unknown internal error occurred")
                    .with_note(format!("panicked at {}", location))
                    .with_note(payload.unwrap_or(String::from("<unknown>")));
                diagnostic = match backtrace.status() {
                    BacktraceStatus::Captured => {
                        diagnostic.with_note(format!("Backtrace:\n{}", backtrace))
                    }
                    BacktraceStatus::Disabled => {
                        diagnostic.with_note("Enable backtrace with RUST_BACKTRACE=1")
                    }
                    _ => diagnostic,
                };
                diagnostic
            }),
        );
        let mut settings = CodespanSettings::stderr(
            Config {
                display_style: DisplayStyle::Medium,
                ..Config::default()
            },
            ColorChoice::AlwaysAnsi,
        );
        _ = report.emit(&mut settings, &Global);
    }

    fn with_schedule(&mut self, label: impl ScheduleLabel, callback: impl FnOnce(&mut Schedule)) {
        let mut schedules = self.engine_world.resource_mut::<Schedules>();
        callback(schedules.entry(label))
    }

    pub fn add_systems<M>(
        &mut self,
        schedule_label: impl ScheduleLabel,
        config: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) {
        self.with_schedule(schedule_label, |schedule| {
            schedule.add_systems(config);
        })
    }

    pub fn add_observer<E: Event, B: Bundle, M>(
        &mut self,
        system: impl IntoObserverSystem<E, B, M>,
    ) {
        self.engine_world.add_observer(system);
    }

    pub fn set_schedule_executor_kind(
        &mut self,
        schedule_label: impl ScheduleLabel,
        kind: ExecutorKind,
    ) {
        self.with_schedule(schedule_label, |schedule| {
            schedule.set_executor_kind(kind);
        })
    }

    pub fn spawn_entity(&mut self, bundle: impl Bundle) -> EntityWorldMut<'_> {
        self.engine_world.spawn(bundle)
    }

    pub fn insert_resource(&mut self, value: impl Resource) {
        self.engine_world.insert_resource(value);
    }

    pub fn init_resource<T: Resource + FromWorld>(&mut self) {
        self.engine_world.init_resource::<T>();
    }
}

impl SubEngine {
    pub fn new() -> Self {
        Self {
            inner: Engine::new(),
        }
    }

    pub fn resource<T>(&self) -> &T
    where
        T: Resource,
    {
        self.engine_world.resource()
    }
}

impl Deref for SubEngine {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<P> Deref for PhaseEngine<'_, P> {
    type Target = Engine;

    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}

impl<P> DerefMut for PhaseEngine<'_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}

#[cfg(test)]
mod tests {
    use crate::define_phase;
    use crate::engine::{Engine, PhaseEngine};
    use bevy_ecs::prelude::{Res, ResMut, Resource};
    use bevy_ecs::schedule::IntoScheduleConfigs;
    use std::hash::Hash;

    #[derive(Debug, Resource)]
    struct Counter(usize);

    fn inc_counter(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    fn check_counter(value: usize) -> impl FnMut(Res<Counter>) {
        move |cnt: Res<Counter>| assert_eq!(cnt.0, value)
    }

    define_phase!(
        phase A[ALabel] { counter: usize }
        fn build(self, engine: &mut PhaseEngine<Self>) {
            engine.add_systems((check_counter(self.counter), inc_counter, check_counter(self.counter + 1)).chain());
        }
    );

    define_phase!(
        phase B[BLabel] { counter: usize }
        fn build(self, engine: &mut PhaseEngine<Self>) {
            engine.add_systems((check_counter(self.counter), inc_counter, check_counter(self.counter + 1)).chain());
        }
    );

    #[inline]
    fn test(configuration: impl FnOnce(&mut Engine)) {
        let mut engine = Engine::new();

        engine.insert_resource(Counter(0));

        configuration(&mut engine);
        engine.run().unwrap();
    }

    #[test]
    fn test_one_phase() {
        test(|e| {
            e.install(A { counter: 0 });
        })
    }

    #[test]
    fn test_two_parallel_phases() {
        test(|e| {
            e.install(A { counter: 1 });
            e.install(B { counter: 0 });
        })
    }

    #[test]
    fn test_two_consecutive_phases() {
        test(|e| {
            e.install(A { counter: 0 }).install(B { counter: 1 });
        })
    }

    #[test]
    fn test_two_same_parallel_phases() {
        test(|e| {
            e.install(A { counter: 1 });
            e.install(A { counter: 0 });
        })
    }

    #[test]
    #[should_panic = "system set `ALabel` has been told to run before itself"]
    fn test_two_same_consecutive_phases() {
        test(|e| {
            e.install(A { counter: 0 }).install(A { counter: 0 });
        })
    }

    #[test]
    #[should_panic = "system set `ALabel` must run before itself"]
    fn test_phases_chaining() {
        test(|e| {
            e.install(A { counter: 0 })
                .install(B { counter: 0 })
                .install(A { counter: 0 });
        })
    }

    #[test]
    fn test_inline_phases() {
        test(|e| {
            e.install_inline_phase(|e| {
                e.add_systems((inc_counter, check_counter(2)).chain());
            });

            e.install_inline_phase(|e| {
                e.install(B { counter: 0 });
            })
        })
    }
}
