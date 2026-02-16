mod task_pool {
    //! Belongs to bevy 0.17.2: https://github.com/bevyengine/bevy/blob/release-0.17.2/crates/bevy_app/src/task_pool_plugin.rs

    use crate::engine::{Engine, Plugin};
    use kodept_ecs::tasks::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool, TaskPoolBuilder};
    use std::fmt::Debug;
    use std::sync::Arc;
    use tracing::trace;

    /// Setup of default task pools: [`AsyncComputeTaskPool`], [`ComputeTaskPool`], [`IoTaskPool`].
    #[derive(Default)]
    pub struct TaskPoolPlugin {
        /// Options for the [`TaskPool`](bevy_tasks::TaskPool) created at application start.
        pub task_pool_options: TaskPoolOptions,
    }

    impl Plugin for TaskPoolPlugin {
        fn build(self, _: &mut Engine) {
            self.task_pool_options.create_default_pools();
        }
    }

    /// Defines a simple way to determine how many threads to use given the number of remaining cores
    /// and number of total cores
    #[derive(Clone)]
    pub struct TaskPoolThreadAssignmentPolicy {
        /// Force using at least this many threads
        pub min_threads: usize,
        /// Under no circumstance use more than this many threads for this pool
        pub max_threads: usize,
        /// Target using this percentage of total cores, clamped by `min_threads` and `max_threads`. It is
        /// permitted to use 1.0 to try to use all remaining threads
        pub percent: f32,
        /// Callback that is invoked once for every created thread as it starts.
        /// This configuration will be ignored under wasm platform.
        pub on_thread_spawn: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
        /// Callback that is invoked once for every created thread as it terminates
        /// This configuration will be ignored under wasm platform.
        pub on_thread_destroy: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    }

    impl Debug for TaskPoolThreadAssignmentPolicy {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("TaskPoolThreadAssignmentPolicy")
                .field("min_threads", &self.min_threads)
                .field("max_threads", &self.max_threads)
                .field("percent", &self.percent)
                .finish()
        }
    }

    impl TaskPoolThreadAssignmentPolicy {
        /// Determine the number of threads to use for this task pool
        fn get_number_of_threads(&self, remaining_threads: usize, total_threads: usize) -> usize {
            assert!(self.percent >= 0.0);
            let proportion = total_threads as f32 * self.percent;
            let mut desired = proportion as usize;

            // Equivalent to round() for positive floats without libm requirement for
            // no_std compatibility
            if proportion - desired as f32 >= 0.5 {
                desired += 1;
            }

            // Limit ourselves to the number of cores available
            desired = desired.min(remaining_threads);

            // Clamp by min_threads, max_threads. (This may result in us using more threads than are
            // available, this is intended. An example case where this might happen is a device with
            // <= 2 threads.
            desired.clamp(self.min_threads, self.max_threads)
        }
    }

    /// Helper for configuring and creating the default task pools. For end-users who want full control,
    /// set up [`TaskPoolPlugin`]
    #[derive(Clone, Debug)]
    pub struct TaskPoolOptions {
        /// If the number of physical cores is less than `min_total_threads`, force using
        /// `min_total_threads`
        pub min_total_threads: usize,
        /// If the number of physical cores is greater than `max_total_threads`, force using
        /// `max_total_threads`
        pub max_total_threads: usize,

        /// Used to determine number of IO threads to allocate
        pub io: TaskPoolThreadAssignmentPolicy,
        /// Used to determine number of async compute threads to allocate
        pub async_compute: TaskPoolThreadAssignmentPolicy,
        /// Used to determine number of compute threads to allocate
        pub compute: TaskPoolThreadAssignmentPolicy,
    }

    impl Default for TaskPoolOptions {
        fn default() -> Self {
            TaskPoolOptions {
                // By default, use however many cores are available on the system
                min_total_threads: 1,
                max_total_threads: usize::MAX,

                // Use 25% of cores for IO, at least 1, no more than 4
                io: TaskPoolThreadAssignmentPolicy {
                    min_threads: 1,
                    max_threads: 4,
                    percent: 0.25,
                    on_thread_spawn: None,
                    on_thread_destroy: None,
                },

                // Use 25% of cores for async compute, at least 1, no more than 4
                async_compute: TaskPoolThreadAssignmentPolicy {
                    min_threads: 1,
                    max_threads: 4,
                    percent: 0.25,
                    on_thread_spawn: None,
                    on_thread_destroy: None,
                },

                // Use all remaining cores for compute (at least 1)
                compute: TaskPoolThreadAssignmentPolicy {
                    min_threads: 1,
                    max_threads: usize::MAX,
                    percent: 1.0, // This 1.0 here means "whatever is left over"
                    on_thread_spawn: None,
                    on_thread_destroy: None,
                },
            }
        }
    }

    impl TaskPoolOptions {
        /// Create a configuration that forces using the given number of threads.
        pub fn with_num_threads(thread_count: usize) -> Self {
            TaskPoolOptions {
                min_total_threads: thread_count,
                max_total_threads: thread_count,
                ..Default::default()
            }
        }

        /// Inserts the default thread pools into the given resource map based on the configured values
        pub fn create_default_pools(&self) {
            let total_threads = std::thread::available_parallelism()
                .map_or(1, |it| it.get())
                .clamp(self.min_total_threads, self.max_total_threads);
            trace!("Assigning {total_threads} cores to default task pools");

            let mut remaining_threads = total_threads;

            {
                // Determine the number of IO threads we will use
                let io_threads = self
                    .io
                    .get_number_of_threads(remaining_threads, total_threads);

                trace!("IO Threads: {io_threads}");
                remaining_threads = remaining_threads.saturating_sub(io_threads);

                IoTaskPool::get_or_init(|| {
                    let builder = TaskPoolBuilder::default()
                        .num_threads(io_threads)
                        .thread_name("IO Task Pool".to_string());

                    let builder = {
                        let mut builder = builder;
                        if let Some(f) = self.io.on_thread_spawn.clone() {
                            builder = builder.on_thread_spawn(move || f());
                        }
                        if let Some(f) = self.io.on_thread_destroy.clone() {
                            builder = builder.on_thread_destroy(move || f());
                        }
                        builder
                    };

                    builder.build()
                });
            }

            {
                // Determine the number of async compute threads we will use
                let async_compute_threads = self
                    .async_compute
                    .get_number_of_threads(remaining_threads, total_threads);

                trace!("Async Compute Threads: {async_compute_threads}");
                remaining_threads = remaining_threads.saturating_sub(async_compute_threads);

                AsyncComputeTaskPool::get_or_init(|| {
                    let builder = TaskPoolBuilder::default()
                        .num_threads(async_compute_threads)
                        .thread_name("Async Compute Task Pool".to_string());

                    let builder = {
                        let mut builder = builder;
                        if let Some(f) = self.async_compute.on_thread_spawn.clone() {
                            builder = builder.on_thread_spawn(move || f());
                        }
                        if let Some(f) = self.async_compute.on_thread_destroy.clone() {
                            builder = builder.on_thread_destroy(move || f());
                        }
                        builder
                    };

                    builder.build()
                });
            }

            {
                // Determine the number of compute threads we will use
                // This is intentionally last so that an end user can specify 1.0 as the percent
                let compute_threads = self
                    .compute
                    .get_number_of_threads(remaining_threads, total_threads);

                trace!("Compute Threads: {compute_threads}");

                ComputeTaskPool::get_or_init(|| {
                    let builder = TaskPoolBuilder::default()
                        .num_threads(compute_threads)
                        .thread_name("Compute Task Pool".to_string());

                    let builder = {
                        let mut builder = builder;
                        if let Some(f) = self.compute.on_thread_spawn.clone() {
                            builder = builder.on_thread_spawn(move || f());
                        }
                        if let Some(f) = self.compute.on_thread_destroy.clone() {
                            builder = builder.on_thread_destroy(move || f());
                        }
                        builder
                    };

                    builder.build()
                });
            }
        }
    }
}

pub(super) mod instrument {
    use crate::engine::Phase;
    use crate::engine::inner_set::InnerSet;
    use kodept_ecs::exported::bevy_ecs;
    use kodept_ecs::resource::Resource;
    use kodept_ecs::schedule::{
        Chain, GraphInfo, InternedSystemSet, IntoScheduleConfigs, IntoSystemSet, Schedulable,
        ScheduleConfigs,
    };
    use kodept_ecs::system::{If, Res, ResMut, ScheduleSystem};
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU16;
    use std::time::{Duration, Instant};
    use tracing::{Level, debug, error, info, trace, warn};

    #[derive(Debug, Resource, Default)]
    pub struct Timings {
        starts: HashMap<u16, Instant>,
    }

    #[derive(Debug, Resource)]
    pub struct TimingsOptions {
        log_level: Level,
    }

    impl Default for TimingsOptions {
        fn default() -> Self {
            Self {
                log_level: Level::DEBUG,
            }
        }
    }

    fn pick_appropriate_suffix(dur: Duration) -> (f64, &'static str) {
        if dur < Duration::from_millis(1) {
            (dur.as_secs_f64() * 1e6, "μs")
        } else if dur < Duration::from_secs(1) {
            (dur.as_secs_f64() * 1000.0, "ms")
        } else if dur < Duration::from_secs(60) {
            (dur.as_secs_f64(), "s")
        } else if dur < Duration::from_secs(3600) {
            (dur.as_secs_f64() / 60.0, "min")
        } else {
            (dur.as_secs_f64() / 3600.0, "h")
        }
    }

    #[must_use]
    pub(crate) fn instrument<P: Phase>(name: &'static str) -> ScheduleConfigs<ScheduleSystem> {
        static GENERATOR: AtomicU16 = AtomicU16::new(0);
        let id = GENERATOR.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let before = move |mut timings: If<ResMut<Timings>>| {
            timings.starts.insert(id, Instant::now());
        };
        let after = move |timings: If<Res<Timings>>, options: Option<Res<TimingsOptions>>| {
            if let Some(instant) = timings.starts.get(&id) {
                let (duration, suffix) = pick_appropriate_suffix(instant.elapsed());
                match options
                    .as_ref()
                    .map_or(TimingsOptions::default().log_level, |it| it.log_level)
                {
                    Level::TRACE => trace!("{name} finished after {duration:.3}{suffix}"),
                    Level::DEBUG => debug!("{name} finished after {duration:.3}{suffix}"),
                    Level::INFO => info!("{name} finished after {duration:.3}{suffix}"),
                    Level::WARN => warn!("{name} finished after {duration:.3}{suffix}"),
                    Level::ERROR => error!("{name} finished after {duration:.3}{suffix}"),
                }
            }
        };

        let label = P::Set::default();
        IntoScheduleConfigs::into_configs(
            (
                before.before_ignore_deferred(InnerSet::<P::Set>::new()),
                after.after_ignore_deferred(InnerSet::<P::Set>::new()),
            )
                .in_set(label.into_system_set()),
        )
    }
}

mod inject_resources {
    use crate::engine::{Phase, PhaseEngine, SubEngine};
    use kodept_ecs::entity::Entity;
    use kodept_ecs::exported::bevy_ecs;
    use kodept_ecs::query::With;
    use kodept_ecs::schedule::SystemSet;
    use kodept_ecs::system::{InMut, IntoSystem, ReadOnlySystem};
    use kodept_ecs::world::World;
    use std::fmt::Debug;
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;
    use tracing::error;

    #[derive(SystemSet)]
    pub struct InjectResourcesPhaseLabel<F>(PhantomData<fn() -> F>);

    pub struct InjectResourcesPhase<'a, F>(F, PhantomData<fn() -> &'a ()>);

    impl<'a, F> InjectResourcesPhase<'a, F> {
        pub fn new<M>(
            inject_system: impl IntoSystem<InMut<'a, SubEngine>, (), M, System = F>,
        ) -> Self {
            InjectResourcesPhase(IntoSystem::into_system(inject_system), PhantomData)
        }
    }

    impl<'a, F> Phase for InjectResourcesPhase<'a, F>
    where
        F: 'static,
        F: ReadOnlySystem<In = InMut<'a, SubEngine>, Out = ()>,
    {
        type Set = InjectResourcesPhaseLabel<F>;

        fn build(mut self, engine: &mut PhaseEngine<Self>) {
            engine.instrumented = false;
            self.0.initialize(&mut engine.engine_world);
            engine.add_systems(system.with_input(self.0));
        }
    }

    fn system<'a, F>(InMut(inject_system): InMut<F>, world: &mut World)
    where
        F: ReadOnlySystem<In = InMut<'a, SubEngine>, Out = ()>,
    {
        let mut sub_engines_query_state = world.query_filtered::<Entity, With<SubEngine>>();
        let sub_engines_entities: Vec<_> =
            sub_engines_query_state.query(world).into_iter().collect();

        for entity in sub_engines_entities {
            let mut sub_engine = world.entity_mut(entity).take::<SubEngine>().unwrap();
            if let Err(e) = inject_system.run_readonly(&mut sub_engine, world) {
                error!("Cannot run inject system: {e}");
            }
            world.entity_mut(entity).insert(sub_engine);
        }
    }

    impl<F> Debug for InjectResourcesPhaseLabel<F> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("InjectResourcesPhaseLabel").finish()
        }
    }

    impl<F> Clone for InjectResourcesPhaseLabel<F> {
        fn clone(&self) -> Self {
            Self(PhantomData)
        }
    }

    impl<F> Copy for InjectResourcesPhaseLabel<F> {}

    impl<F> PartialEq for InjectResourcesPhaseLabel<F> {
        fn eq(&self, _: &Self) -> bool {
            true
        }
    }

    impl<F> Eq for InjectResourcesPhaseLabel<F> {}

    impl<F> Hash for InjectResourcesPhaseLabel<F> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    impl<F> Default for InjectResourcesPhaseLabel<F> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }
}

pub use inject_resources::{InjectResourcesPhase, InjectResourcesPhaseLabel};
pub use instrument::{Timings, TimingsOptions};
pub use task_pool::{TaskPoolOptions, TaskPoolPlugin, TaskPoolThreadAssignmentPolicy};
