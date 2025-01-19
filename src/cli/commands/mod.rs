use crate::cli::commands::execute::Execute;
use crate::cli::commands::graph::Graph;
use crate::cli::commands::inspect::InspectParser;
use clap::Subcommand;

mod execute;
mod graph;
mod inspect;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Output AST in .dot format
    Graph(Graph),
    /// Output parsing process files
    InspectParser(InspectParser),
    /// Run type checker
    Execute(Execute),
}
