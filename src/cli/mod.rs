use crate::cli::commands::Commands;
use crate::cli::common::Kodept;
use bevy_ecs::prelude::Resource;
use clap::Parser;
use kodept::codespan_settings::Reports;
use kodept_core::file_name::FileName;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;
use std::borrow::Cow;
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::PathBuf;

pub(crate) mod commands;
pub mod common;
pub mod configs;
pub mod utils;

pub struct CliArgumentsPlugin;

#[derive(Debug, Resource)]
pub struct OutputConfig {
    path: PathBuf,
}

impl Plugin for CliArgumentsPlugin {
    fn build(self, app: &mut Frontend) {
        let args = Kodept::parse();

        app.insert_resource(OutputConfig { path: args.output });
        app.insert_resource(args.logging);
        app.insert_resource(Reports::from(args.diagnostic_config));

        // delegate initialization to subcommands
        match args.subcommands {
            Commands::Graph(x) => x.build(app),
            Commands::InspectParser(_) => {}
            Commands::Execute(x) => x.build(app),
        };
    }
}

impl OutputConfig {
    pub fn open_file_for_source(
        &self,
        source: FileName,
        extension: impl Into<Cow<'static, str>>,
    ) -> std::io::Result<File> {
        let new_path = source
            .build_file_path()
            .with_extension(extension.into().as_ref());
        let name = new_path.file_name().unwrap();
        self.create_missing_folders()?;
        File::create(self.path.join(name))
    }

    fn create_missing_folders(&self) -> std::io::Result<()> {
        match create_dir_all(&self.path) {
            Err(e) if e.kind() != ErrorKind::AlreadyExists => Err(e),
            _ => Ok(()),
        }
    }
}
