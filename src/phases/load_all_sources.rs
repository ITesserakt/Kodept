use crate::cli::configs::LoadingConfig;
use bevy_ecs::prelude::*;
use kodept::loader::{Loader, LoadingError};
use kodept::source::{SourcesLoadingError, load_each_source};
use kodept::utils::ReportSystemEx;
use kodept_frontend::Either;
use kodept_frontend::engine::{Phase, PhaseEngine, SubEngine};
use kodept_frontend::prelude::CollectedSources;
use std::sync::Arc;

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LoadAllSourcesPhaseSystems;

pub struct LoadAllSourcesPhase {
    pub config: LoadingConfig,
}

impl Phase for LoadAllSourcesPhase {
    type Set = LoadAllSourcesPhaseSystems;

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(system.with_input(self.config).extract_reports());
    }
}

fn system(
    InMut(config): InMut<LoadingConfig>,
    mut commands: Commands,
) -> Result<(), Either<LoadingError, SourcesLoadingError>> {
    let loader = Loader::try_from(&*config).map_err(Either::Left)?;
    let sources = load_each_source(loader).map_err(Either::Right)?;
    let sources = Arc::new(sources);
    let views = sources.collect();

    commands.insert_resource(CollectedSources { inner: sources });
    commands.spawn_batch(views.into_iter().map(move |it| (it, SubEngine::new())));

    Ok(())
}
