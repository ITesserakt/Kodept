use crate::configs::OutputDirectory;
use crate::source::collection::{Reporter, SourceView};
use crate::utils::ReportSystemEx;
use bevy_ecs::prelude::*;
use derive_more::{Display, Error, From};
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report::prelude::{Diagnostic, Severity};
use std::fs::File;

define_phase!(
    pub phase ExportRltPhase[ExportRltPhaseLabel];

    fn build (self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(
            system.extract_reports(),
        );
    }
);

#[derive(Debug, Error, From, Display)]
enum Error {
    #[display("Could not open file to output RLT: {_0}")]
    IO(std::io::Error),
    #[display("Could not serialize RLT into json: {_0}")]
    Serde(serde_json::Error),
}

fn system(
    config: Res<OutputDirectory>,
    source: Res<SourceView>,
    syntax: Res<SyntaxResolver>,
    mut reporter: Reporter,
) -> Result<(), Error> {
    let output_filepath = config.get_path_for_source(source.path(), "rlt.json")?;
    let output_file = File::create(&output_filepath)?;
    serde_json::to_writer(output_file, syntax.root().0)?;

    reporter.report_ad_hoc(|| {
        Diagnostic::new(Severity::Note).with_message(format!(
            "Successfully exported RLT to {}",
            output_filepath.display()
        ))
    });

    Ok(())
}
