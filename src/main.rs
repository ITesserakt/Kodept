use clap::Parser;
#[cfg(feature = "profiler")]
use kodept::profiler::HeapProfiler;
use std::sync::Arc;

use crate::cli::traits::Command;
use cli::common::Kodept;
use kodept::loader::Loader;
use kodept::source_files::SourceFiles;

mod cli;

type WideError = anyhow::Error;

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

    let mut reports = cli_arguments.clone().diagnostic_config.into();
    let loader: Loader = cli_arguments.clone().loading_config.try_into()?;
    let sources = Arc::new(SourceFiles::from_sources(loader.into_sources()));

    let args = cli_arguments.clone();
    let result =
        cli_arguments
            .subcommands
            .exec(sources.clone().into_common_iter(), &mut reports, args);
    reports.consume(&*sources);

    #[cfg(feature = "profiler")]
    {
        HeapProfiler::consume();
    }
    result.map_err(WideError::from)
}
