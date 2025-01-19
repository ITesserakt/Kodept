use crate::cli::commands::Commands;
use crate::cli::common::Kodept;
use clap::Parser;
use kodept::codespan_settings::Reports;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;

pub(crate) mod commands;
pub mod common;
pub mod configs;
pub mod utils;

pub struct CliArgumentsPlugin;

impl Plugin for CliArgumentsPlugin {
    fn build(self, app: &mut Frontend) {
        let args = Kodept::parse();

        app.insert_resource(args.logging);
        app.insert_resource(Reports::from(args.diagnostic_config));

        // delegate initialization to subcommands
        match args.subcommands {
            Commands::Graph(_) => {}
            Commands::InspectParser(_) => {}
            Commands::Execute(x) => x.build(app),
        };
    }
}
