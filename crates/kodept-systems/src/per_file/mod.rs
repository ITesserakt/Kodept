use bevy_ecs::prelude::*;
use kodept_ast::resource::reflection::DebugRegistry;
use kodept_frontend::engine::{reporter, Phase, SubEngine};
use kodept_frontend::engine::utils::{InjectResourcesPhase, Timings};

mod build_ast;
mod parse_source;
mod export_rlt;
mod ast_shenanigans;

pub mod prelude {
    pub use super::build_ast::{BuildAstPhase, BuildAstPhaseLabel};
    pub use super::export_rlt::{ExportRltPhase, ExportRltPhaseLabel};
    pub use super::parse_source::{ParseSourcePhase, ParseSourcePhaseLabel};
    pub use super::ast_shenanigans::*;
}

pub fn inject_common_resources_phase() -> impl Phase {
    InjectResourcesPhase::new(inject_common_resources)
}

fn inject_common_resources(
    InMut(engine): InMut<SubEngine>,
    timings: Option<Res<Timings>>,
    report_settings: Option<Res<reporter::Settings>>,
    registry: Option<Res<DebugRegistry>>
) {
    if timings.is_some() {
        engine.init_resource::<Timings>();
    }
    if let Some(report_settings) = report_settings {
        engine.insert_resource(report_settings.clone())
    }
    if let Some(registry) = registry {
        engine.insert_resource(registry.clone());
    }
}
