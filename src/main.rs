mod cli;
mod stage;

use crate::stage::{
    BuildingAST, BuildingRLT, Emitting, PredefinedTraverseSet, Prepare, Reading, Traversing,
};
use indicatif::{ProgressBar, ProgressStyle};
use kodept::macro_context::ErrorReported;
use rayon::prelude::*;
use std::process::ExitCode;

type WideError = anyhow::Error;

fn main() -> Result<ExitCode, WideError> {
    let p_style = ProgressStyle::with_template(
        "({elapsed_precise}) [{bar:.green/cyan}] ({pos}/{len}) - {msg}",
    )?
    .progress_chars("█▉▊▋▌▍▎▏  ");
    let progress = ProgressBar::new(1).with_style(p_style);

    let (settings, sources) = Prepare.run()?;
    progress.set_length(sources.len() as u64);

    let any_error_reported: Result<_, WideError> = sources
        .into_par_iter()
        .map(|source| Reading.run(source))
        .inspect(|it| match it {
            Ok(s) => progress.set_message(format!("`{}` - read successfully", s.path())),
            Err(_) => progress.set_message("Error"),
        })
        .map_with(settings, |settings, source| {
            let read_source = source?;
            let rlt = match BuildingRLT.run(&read_source, settings) {
                None => return Ok(true),
                Some(x) => x,
            };

            let (mut ast, context) = BuildingAST.run(&read_source, &rlt);
            let set = PredefinedTraverseSet::default();
            let context = Traversing.run(&set, &mut ast, context, &read_source, settings);
            progress.inc(1);
            Ok(Emitting.run(context, &read_source, settings))
        })
        .try_reduce(|| false, |next, acc| Ok(acc | next));
    if any_error_reported? {
        Err(ErrorReported)?;
    }
    Ok(ExitCode::FAILURE)
}