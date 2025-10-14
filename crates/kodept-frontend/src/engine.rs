use crate::engine::reporter::{Settings};
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy_ecs::system::ScheduleSystem;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub mod macros {
    #[macro_export]
    macro_rules! define_phase {
        ($vis:vis phase $name:ident[$label:ident]
            {$($field_vis:vis $field:ident: $ty:ty$(,)?)*}
            fn build($self:ident, $engine:ident: $engine_ty:ty) $build:block
        ) => {
            #[derive(Debug, bevy_ecs::prelude::SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $label;

            pub struct $name {
                $(
                $field_vis $field: $ty
                )*
            }

            impl $crate::engine::Phase for $name {
                type Set = $label;

                fn build($self, $engine: $engine_ty) {
                    $build
                }
            }
        };
        (
            $vis:vis phase $name:ident[$label:ident];
            fn build($self:ident, $engine:ident: $engine_ty:ty) $build:block
        ) => {
            #[derive(Debug, bevy_ecs::prelude::SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $label;
            pub struct $name;

            impl $crate::engine::Phase for $name {
                type Set = $label;

                fn build($self, $engine: $engine_ty) {
                    $build
                }
            }
        }
    }
}

pub mod reporter {
    use crate::prelude::{CollectedSources, Global, Source, SourceView};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::{SystemBuffer, SystemMeta, SystemParam};
    use kodept_report::prelude::*;
    use std::marker::PhantomData;
    use tracing::error;

    enum GenericReport {
        Single(Report),
        Global(Report<()>),
    }

    struct Reports<Impl>(Vec<GenericReport>, PhantomData<fn() -> Impl>);

    impl<Impl> Default for Reports<Impl> {
        fn default() -> Self {
            Self(vec![], PhantomData)
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

                match &mut *settings {
                    Settings::Disabled => self.0.clear(),
                    Settings::Eager(_) => {
                        unreachable!("All reports should have been reported already")
                    }
                    Settings::Lazy(settings) => {
                        for report in self.0.drain(..) {
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
            })
        }
    }

    #[derive(Debug, Resource, Default)]
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
                        .0
                        .push(GenericReport::Single(Report::from_message(
                            *source.id, message,
                        )));
                }
                (Settings::Lazy(_), None, _) => {
                    self.buffer
                        .0
                        .push(GenericReport::Global(Report::from_message((), message)));
                }
                (Settings::Disabled, _, _) => {}
            }
        }

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
}

#[derive(Debug, Resource)]
pub struct PhasesOrder {}

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

pub struct RunAndForget;

pub trait Phase {
    type Set: IntoSystemSet<()> + Default;

    fn build(self, engine: &mut Engine);
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

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedules = Schedules::new();
        schedules
            .entry(Startup)
            .set_executor_kind(ExecutorKind::SingleThreaded);

        world.insert_resource(schedules);
        world.init_resource::<Settings>();

        Self {
            engine_world: world,
        }
    }

    pub fn install<P: Phase>(&mut self, phase: P) -> Chaining<'_, P> {
        phase.build(self);
        Chaining {
            engine: self,
            _phantom: PhantomData,
        }
    }

    pub fn run(&mut self) {
        self.engine_world.run_schedule(Startup);
    }

    fn with_schedule(&mut self, label: impl ScheduleLabel, callback: impl FnOnce(&mut Schedule)) {
        let mut schedules = self.engine_world.resource_mut::<Schedules>();
        callback(schedules.entry(label))
    }

    pub fn add_systems<M>(&mut self, config: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        self.with_schedule(Startup, |schedule| {
            schedule.add_systems(config);
        })
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
