use crate::cli::configs::LoadingConfig;
use bevy_ecs::prelude::{Commands, Res};
use kodept::codespan_settings::GlobalReportEmitted;
use kodept::loader::Loader;
use kodept::source_files::{SourceFiles, Sources};
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;
use std::ops::Deref;
use std::sync::Arc;
use kodept_frontend::SystemExt;

pub struct LoadSourcesPlugin;

impl Plugin for LoadSourcesPlugin {
    fn build(self, app: &mut Frontend) {
        app.add_systems(load_sources_system.run_once());
    }
}

fn load_sources_system(config: Res<LoadingConfig>, mut commands: Commands) {
    let loader: Result<Loader, _> = config.deref().try_into();
    let loader = match loader {
        Ok(x) => x,
        Err(e) => {
            commands.send_event(GlobalReportEmitted::new(e));
            return;
        }
    };
    let sources = loader.into_sources();
    let mut source_files = SourceFiles::new();
    for source in sources {
        match source_files.insert(source) {
            Ok(()) => {}
            Err(e) => {
                commands.send_event(GlobalReportEmitted::new(e));
                continue;
            }
        }
    }
    let source_files = Arc::new(source_files);
    let sources = Sources(source_files.clone());
    commands.insert_resource(sources);
    commands.spawn_batch(source_files.collect());
}
