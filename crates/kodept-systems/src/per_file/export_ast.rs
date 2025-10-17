use crate::configs::OutputDirectory;
use crate::source::collection::{Reporter, SourceView};
use crate::utils::ReportSystemEx;
use bevy_ecs::prelude::*;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report::prelude::{Diagnostic, Severity};
use std::fs::File;

define_phase!(
    pub phase ExportAstPhase[ExportAstPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(
            system.extract_reports()
        )
    }
);

fn system(
    config: Res<OutputDirectory>,
    source: Res<SourceView>,
    mut reporter: Reporter,
    world: &World,
) -> Result<(), std::io::Error> {
    let output_filepath = config.get_path_for_source(source.path(), "puml")?;
    let mut output_file = File::create(&output_filepath)?;
    AST::export_dot_in(world, &mut output_file)
        .expect("Some components did not registered still")?;

    reporter.report_ad_hoc(|| {
        Diagnostic::new(Severity::Note).with_message(format!(
            "Successfully exported AST to {}",
            output_filepath.display()
        ))
    });

    Ok(())
}
