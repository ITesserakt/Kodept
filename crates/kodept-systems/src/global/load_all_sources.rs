use crate::loader::{Loader, LoadingError};
use crate::source::{SourcesLoadingError, load_each_source};
use crate::utils::ReportSystemEx;
use kodept_core::either::Either;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::schedule::SystemSet;
use kodept_ecs::system::{Commands, InMut, IntoSystem};
use kodept_frontend::engine::{Phase, PhaseEngine, SubEngine};
use kodept_frontend::prelude::CollectedSources;
use std::sync::Arc;

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LoadAllSourcesPhaseLabel;

pub struct LoadAllSourcesPhase<Config> {
    pub config: Config,
}

impl<C> Phase for LoadAllSourcesPhase<C>
where
    for<'a> Loader: TryFrom<&'a C, Error = LoadingError>,
    C: Send + Sync + 'static,
{
    type Set = LoadAllSourcesPhaseLabel;

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(system.with_input(self.config).extract_reports());
    }
}

fn system<C>(
    InMut(config): InMut<C>,
    mut commands: Commands,
) -> Result<(), Either<LoadingError, SourcesLoadingError>>
where
    for<'a> Loader: TryFrom<&'a C, Error = LoadingError>,
{
    let loader = Loader::try_from(&*config).map_err(Either::Left)?;
    let sources = load_each_source(loader).map_err(Either::Right)?;
    let sources = Arc::new(sources);
    let views = sources.collect();

    commands.insert_resource(CollectedSources { inner: sources });
    commands.spawn_batch(views.into_iter().map(move |it| (it, SubEngine::new())));

    Ok(())
}
