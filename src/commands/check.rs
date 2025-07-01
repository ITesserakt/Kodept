use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::Command;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept_ast::interaction::Interaction as Ctx;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_frontend::Execution;
use kodept_interaction::lint::{
    DebugScopesLint, LintDescriptor, RLTLinkLint, ShowLints, SingleModuleWithBrackets,
};
use kodept_interaction::prelude::{
    install_reporting_support, ExtractSymbolsPass, Phases, ReferenceResolverPass, ScopeBuildingPass,
};
use kodept_interaction::Interaction;
use std::borrow::Cow;
use std::ops::ControlFlow::Continue;
use std::time::{Duration, Instant};
use tracing::{debug, enabled, error_span, info, info_span, Level};

#[derive(Debug, Parser)]
pub struct Check {
    /// Measure duration of different stages
    #[arg(short, long, action, default_value_t = false)]
    timings: bool,
    /// Enable some lints during analysis that are disabled by default
    #[arg(long)]
    enabled_lints: Vec<String>,

    #[command(flatten, next_help_heading = "Loading options")]
    loading_config: LoadingConfig,
    #[command(flatten, next_help_heading = "Parsing options")]
    parsing_config: ParsingConfig,
}

impl Command for Check {
    fn exec(self, reports: GlobalReports, _: OutputConfig) -> Execution<()> {
        let (sources, reports) = self.timings_block("Source loading", || {
            get_all_sources(&self.loading_config, reports)
        })?;
        for source in sources.collect() {
            let _guard = error_span!("", source = %source.path()).entered();
            let rlt = self.timings_block("RLT building", || {
                get_rlt(&self.parsing_config, &source, &reports)
            })?;
            let mut ast = self.timings_block("AST building", || build_ast(&source, rlt));

            self.interaction_block("Linting (first pass)", &mut ast, |ctx| {
                install_reporting_support(ctx, {
                    let reports = reports.clone();
                    move |r| reports.insert(r)
                });
                self.install_lints(ctx);

                Phases::install(ctx);
                ScopeBuildingPass::install(ctx);
                ExtractSymbolsPass::install(ctx);
                ReferenceResolverPass::install(ctx);

                for _ in 0..10 {
                    ctx.launch();
                    debug!("Pass completed");
                }
            });
        }
        Continue(())
    }
}

impl Check {
    fn interaction_block<T>(
        &self,
        name: impl Into<Cow<'static, str>>,
        ast: &mut AST,
        f: impl FnOnce(&mut Ctx) -> T,
    ) -> T {
        let mut ctx = ast.interact();

        self.timings_block(name, || f(&mut ctx))
    }

    fn install_lints(&self, ctx: &mut Ctx) {
        SingleModuleWithBrackets::install(ctx);
        RLTLinkLint::install(ctx);
        ShowLints::install(ctx);
        DebugScopesLint::install(ctx);

        ctx.immediate_exclusive(|w| {
            let mut lint_query = w.query::<&mut LintDescriptor>();

            for mut descriptor in lint_query.iter_mut(w) {
                let search = self
                    .enabled_lints
                    .iter()
                    .find(|name| *name == descriptor.name());

                if search.is_some() {
                    descriptor.enabled = true;
                }
            }
        });
    }

    fn timings_block<'a, T>(&self, name: impl Into<Cow<'a, str>>, f: impl FnOnce() -> T) -> T {
        let span = info_span!("timings-block");
        let _guard = span.enter();
        if self.timings && enabled!(Level::INFO) {
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
