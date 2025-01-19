use crate::actions::build_ast::BuildASTPlugin;
use crate::actions::load_sources::LoadSourcesPlugin;
use crate::actions::parse_sources::ParseSourcesPlugin;
use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::OutputConfig;
use bevy_ecs::prelude::{Entity, IntoSystem, Populated, Res, With};
use bevy_hierarchy::{Children, HierarchyQueryExt};
use clap::Parser;
use codespan_reporting::files::Files;
use kodept::codespan_settings::Reports;
use kodept::source_files::SourceView;
use kodept_ast::properties::Node;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;
use std::io::Write;

#[derive(Parser, Debug, Clone)]
pub struct Graph {
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
}

impl Plugin for Graph {
    fn build(self, world: &mut Frontend) {
        world.insert_resource(self.parsing_config);
        world.insert_resource(self.loading_config);

        world
            .add_plugin(LoadSourcesPlugin)
            .add_plugin(ParseSourcesPlugin)
            .add_plugin(BuildASTPlugin);

        world.on_shutdown(build_dot_system.pipe(Reports::stop_and_report));
    }
}

fn build_dot_system(
    nodes: Populated<&Children>,
    nodes_with_children: Populated<(Entity, &Children), With<Node>>,
    kinds: Populated<&Node>,
    roots: Populated<(Entity, &SourceView)>,
    config: Res<OutputConfig>,
) -> std::io::Result<()> {
    for (root, source) in roots.into_iter() {
        let mut output = config.open_file_for_source(source.name(()).unwrap(), "kd.dot")?;

        writeln!(output, "digraph {{")?;
        for node in nodes.iter_descendants(root) {
            writeln!(
                output,
                "\t{} [ label = \"{} [{}v{}]\" ]",
                node.to_bits(),
                kinds.get(node).unwrap().kind,
                node.index(),
                node.generation()
            )?;
        }
        for (this, children) in nodes_with_children.iter() {
            for child in children {
                writeln!(output, "\t{} -> {}", this.to_bits(), child.to_bits())?;
            }
        }
        writeln!(output, "}}")?;
    }
    Ok(())
}
