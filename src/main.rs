use clap::Parser;
#[cfg(feature = "profiler")]
use kodept::profiler::HeapProfiler;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::cli::traits::Command;
use cli::common::Kodept;
use kodept::loader::Loader;
use kodept::source_files::SourceFiles;
use kodept_core::code_source::CodeSource;

mod cli;

type WideError = anyhow::Error;

#[cfg(not(feature = "parallel"))]
fn sources_iter(loader: Loader) -> impl Iterator<Item = CodeSource> {
    loader.into_sources().into_iter()
}

#[cfg(feature = "parallel")]
fn sources_iter(loader: Loader) -> impl ParallelIterator<Item = CodeSource> {
    loader.into_sources().into_par_iter()
}

fn main() -> Result<(), WideError> {
    #[cfg(feature = "profiler")]
    {
        HeapProfiler::install();
        ctrlc::set_handler(|| {
            HeapProfiler::consume();
            std::process::exit(0);
        })?;
    }

    let cli_arguments: Kodept = Kodept::parse();
    tracing_subscriber::fmt()
        .with_max_level(cli_arguments.level())
        .init();

    let settings = cli_arguments.clone().diagnostic_config.into();
    let loader: Loader = cli_arguments.clone().loading_config.try_into()?;
    let sources = SourceFiles::from_sources(loader.into_sources());

    let args = cli_arguments.clone();
    let result = cli_arguments
        .subcommands
        .exec(sources.into_common_iter(), settings, args);

    #[cfg(feature = "profiler")]
    {
        HeapProfiler::consume();
    }
    result.map_err(WideError::from)
}
