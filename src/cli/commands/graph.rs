use crate::cli::configs::{LoadingConfig, ParsingConfig};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Graph {
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
}
