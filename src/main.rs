use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use kodept::macro_context::ErrorReported;

use crate::stage::{
    BuildingAST, BuildingRLT, Emitting, PredefinedTraverseSet, Prepare, Reading, Traversing,
};

mod cli;
mod stage;

type WideError = anyhow::Error;

fn main() -> Result<(), WideError> {
    let (settings, sources) = Prepare.run()?;

    let any_error_reported: Result<_, WideError> = sources
        .into_par_iter()
        .map(|source| Reading.run(source))
        .map_with(settings, |settings, source| {
            let read_source = source?;
            let rlt = match BuildingRLT.run(&read_source, settings) {
                None => return Ok(true),
                Some(x) => x,
            };

            let context = BuildingAST.run(&read_source, &rlt);
            let mut set = PredefinedTraverseSet::default();
            let context = Traversing.run(&mut set, context, &read_source, settings);
            Ok(Emitting.run(context, &read_source, settings))
        })
        .try_reduce(|| false, |next, acc| Ok(acc | next));
    if any_error_reported? {
        Err(ErrorReported)?;
    }
    Ok(())
}
