use bevy_ecs::prelude::*;
use kodept_frontend::engine::utils::{InjectResourcesPhase, Timings};
use kodept_frontend::engine::{Phase, SubEngine, reporter};

mod ast_shenanigans;
mod build_ast;
mod parse_source;

pub mod prelude {
    pub use super::ast_shenanigans::*;
    pub use super::build_ast::{BuildAstPhase, BuildAstPhaseLabel};
    pub use super::parse_source::{ParseSourcePhase, ParseSourcePhaseLabel};
}

pub fn inject_common_resources_phase() -> impl Phase {
    InjectResourcesPhase::new(inject_common_resources)
}

fn inject_common_resources(
    InMut(engine): InMut<SubEngine>,
    timings: Option<Res<Timings>>,
    report_settings: Option<Res<reporter::Settings>>,
) {
    if timings.is_some() {
        engine.init_resource::<Timings>();
    }
    if let Some(report_settings) = report_settings {
        engine.insert_resource(report_settings.clone())
    }
}
