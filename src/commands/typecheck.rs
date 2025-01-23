use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::Command;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept_ast::interaction::Interaction as Ctx;
use kodept_frontend::Execution;
use kodept_interaction::lint::module::SingleModuleWithBrackets;
use kodept_interaction::report::{ASTExt, FileDescriptor};
use kodept_interaction::Interaction;
use std::borrow::Cow;
use std::ops::ControlFlow::Continue;
use std::time::{Duration, Instant};
use tracing::{info, info_span};

#[derive(Debug, Parser)]
pub struct TypeCheck {
    /// Measure duration of different stages
    #[arg(short, long, action, default_value_t = false)]
    timings: bool,

    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
}

impl Command for TypeCheck {
    fn exec(self, reports: GlobalReports, _: OutputConfig) -> Execution<()> {
        let (sources, reports) = self.timings_block("Source loading", || {
            get_all_sources(&self.loading_config, reports)
        })?;
        for source in sources.collect() {
            let rlt = self.timings_block("RLT building", || {
                get_rlt(&self.parsing_config, &source, &reports)
            })?;
            let mut ast = self.timings_block("AST building", || build_ast(&source, rlt));

            ast.prepare_reporting(FileDescriptor {
                id: *source.id,
                file_name: source.path().clone(),
            });

            let mut lints_interaction = ast.interact();
            install_lints(&mut lints_interaction);
            self.timings_block("Linting (first pass)", || {
                lints_interaction.launch();
            });
            ast.extract_reports(|it| reports.insert(it));
        }
        Continue(())
    }
}

fn install_lints(ctx: &mut Ctx) {
    SingleModuleWithBrackets::install(ctx);
}

impl TypeCheck {
    fn timings_block<'a, T>(&self, name: impl Into<Cow<'a, str>>, f: impl FnOnce() -> T) -> T {
        let span = info_span!("timings-block");
        let _guard = span.enter();
        if self.timings {
            let now = Instant::now();
            let result = f();
            let elapsed = now.elapsed();
            let (value, suffix) = pick_appropriate_suffix(elapsed);
            info!("{} finished after {:.3}{}", name.into(), value, suffix);
            result
        } else {
            f()
        }
    }
}

fn pick_appropriate_suffix(dur: Duration) -> (f64, &'static str) {
    if dur < Duration::from_millis(1) {
        (dur.as_secs_f64() * 1e6, "μs")
    } else if dur < Duration::from_secs(1) {
        (dur.as_secs_f64() * 1000.0, "ms")
    } else if dur < Duration::from_secs(60) {
        (dur.as_secs_f64(), "s")
    } else if dur < Duration::from_secs(3600) {
        (dur.as_secs_f64() / 60.0, "min")
    } else {
        (dur.as_secs_f64() / 3600.0, "h")
    }
}
