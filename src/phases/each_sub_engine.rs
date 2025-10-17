use bevy_ecs::prelude::*;
use kodept::source::collection::SourceView;
use kodept_frontend::engine::{Engine, Phase, PhaseEngine, SubEngine};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use kodept::utils::ReportSystemEx;
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, MessageBehaviour, Severity};

struct CannotProceed;

impl IntoSpannedReportMessage for CannotProceed {
    type Message = Diagnostic;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Cannot process input files")
    }

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Error).with_message("Cannot proceed")
    }
}

#[derive(SystemSet)]
pub struct EachSubEnginePhaseSystems<F>(PhantomData<fn() -> F>);

pub struct EachSubEnginePhase<F>(F);

impl<F> EachSubEnginePhase<F> {
    pub fn new(configuration: F) -> Self
    where
        F: Fn(&mut Engine) + Send + Sync + 'static,
    {
        EachSubEnginePhase(configuration)
    }
}

impl<F: 'static> Phase for EachSubEnginePhase<F>
where
    F: Fn(&mut Engine) + Send + Sync,
{
    type Set = EachSubEnginePhaseSystems<F>;

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.instrumented = false;
        engine.add_systems(system.with_input(self.0).extract_reports())
    }
}

fn system<F>(InMut(configuration): InMut<F>, mut sub_engines: Query<(&SourceView, &mut SubEngine)>) -> Result<(), CannotProceed>
where
    F: Fn(&mut Engine) + Send + Sync + 'static,
{
    let any_stopped = AtomicBool::new(false);
    sub_engines.par_iter_mut().for_each(|(source, mut engine)| {
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

impl<F> Debug for EachSubEnginePhaseSystems<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EachSubEnginePhaseSystems").finish()
    }
}

impl<F> Clone for EachSubEnginePhaseSystems<F> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<F> Copy for EachSubEnginePhaseSystems<F> {}

impl<F> PartialEq for EachSubEnginePhaseSystems<F> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<F> Hash for EachSubEnginePhaseSystems<F> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<F> Default for EachSubEnginePhaseSystems<F> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<F> Eq for EachSubEnginePhaseSystems<F> {}
