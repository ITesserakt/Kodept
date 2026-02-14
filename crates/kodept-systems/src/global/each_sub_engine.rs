use crate::source::collection::SourceView;
use crate::utils::ReportSystemEx;
use bevy_ecs::prelude::*;
use kodept_frontend::engine::{Phase, PhaseEngine, SubEngine};
use kodept_report_macros::Report;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::error_span;

#[derive(Debug, Report)]
#[severity("error")]
#[message("Aborting due to previous errors")]
#[fail_fast("Cannot process input files")]
struct CannotProceed;

#[derive(SystemSet)]
pub struct EachSubEnginePhaseLabel<F>(PhantomData<fn() -> F>);

pub struct EachSubEnginePhase<F>(F);

impl<F> EachSubEnginePhase<F> {
    pub fn new(configuration: F) -> Self
    where
        F: Fn(&mut SubEngine) + Send + Sync + 'static,
    {
        EachSubEnginePhase(configuration)
    }
}

impl<F: 'static> Phase for EachSubEnginePhase<F>
where
    F: Fn(&mut SubEngine) + Send + Sync,
{
    type Set = EachSubEnginePhaseLabel<F>;

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.instrumented = false;
        engine.add_systems(system.with_input(self.0).extract_reports())
    }
}

fn system<F>(
    InMut(configuration): InMut<F>,
    mut sub_engines: Query<(&SourceView, &mut SubEngine)>,
) -> Result<(), CannotProceed>
where
    F: Fn(&mut SubEngine) + Send + Sync + 'static,
{
    let any_stopped = AtomicBool::new(false);
    sub_engines.par_iter_mut().for_each(|(source, mut engine)| {
        let file_name = source.path();
        let span = error_span!("sub_engine", source = %file_name);
        let _guard = span.enter();
        // Update source
        engine.insert_resource(source.clone());
        // Configure engine
        configuration(&mut *engine);
        // Run it!
        if let Err(_) = engine.run() {
            any_stopped.store(true, Ordering::Relaxed);
        }
    });
    if any_stopped.into_inner() {
        Err(CannotProceed)
    } else {
        Ok(())
    }
}

impl<F> Debug for EachSubEnginePhaseLabel<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EachSubEnginePhaseSystems").finish()
    }
}

impl<F> Clone for EachSubEnginePhaseLabel<F> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<F> Copy for EachSubEnginePhaseLabel<F> {}

impl<F> PartialEq for EachSubEnginePhaseLabel<F> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<F> Hash for EachSubEnginePhaseLabel<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<F> Default for EachSubEnginePhaseLabel<F> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<F> Eq for EachSubEnginePhaseLabel<F> {}
