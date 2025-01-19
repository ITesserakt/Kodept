use crate::actions::build_ast::BuildASTPlugin;
use crate::actions::load_sources::LoadSourcesPlugin;
use crate::actions::parse_sources::ParseSourcesPlugin;
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use bevy_ecs::prelude::Commands;
use clap::Args;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::{ExitEvent, Plugin};
use std::num::NonZeroU16;

#[derive(Debug, Args, Clone)]
pub struct Execute {
    /// Specifies maximum number of steps while type checking a function
    #[arg(default_value_t = NonZeroU16::new(256).unwrap(), long = "recursion_depth")]
    type_checking_recursion_depth: NonZeroU16,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
}

impl Plugin for Execute {
    fn build(self, app: &mut Frontend) {
        app.insert_resource(self.loading_config);
        app.insert_resource(self.parsing_config);
        app.add_plugin(LoadSourcesPlugin)
            .add_plugin(ParseSourcesPlugin)
            .add_plugin(BuildASTPlugin);
    }
}

fn send_exit(mut commands: Commands) {
    commands.send_event(ExitEvent);
}
