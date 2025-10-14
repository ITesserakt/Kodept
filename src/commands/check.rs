use crate::cli::configs::{LoadingConfig, ParsingConfig};
use crate::cli::primary::OutputConfig;
use crate::commands::utils::build_ast::build_ast;
use crate::commands::utils::load_source::get_all_sources;
use crate::commands::utils::parse_source::get_rlt;
use crate::commands::Command;
use clap::Parser;
use kodept::report::GlobalReports;
use kodept_frontend::Execution;
use std::borrow::Cow;
use std::ops::ControlFlow::Continue;
use std::time::{Duration, Instant};
use tracing::{enabled, error_span, info, info_span, trace, trace_span, Level};

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
            let mut ast =
                self.timings_block("AST building", || build_ast(&source, rlt, &reports))?;
            let mut ctx = ast.interact();

            // install_reporting_support(&mut ctx, {
            //     let reports = reports.clone();
            //     move |r| reports.insert(r)
            // });

            // install_system_completion_introspection_support(&mut ctx, |event| {
            //     if let Some(reason) = &event.fail_reason {
            //         error!("System {}#{:?} failed: {reason}", event.name, event.id);
            //     } else {
            //         trace!("{}#{:?}", event.name, event.id);
            //     }
            // });

            let mut block_disposal = self.timings_block("Symbols resolution", || {
                // let a = self.install_lints(&mut ctx);
                // let b = ScopeBuildingPass::install(&mut ctx);
                // let c = ExtractSymbolsPass::install(&mut ctx);
                // let d = ReferenceResolverPass::install(&mut ctx);
                // let e = TypeInferPass::install(&mut ctx);

                // How many frames do we actually need?
                for frame in 0..10 {
                    let _guard = trace_span!("pass", frame).entered();
                    ctx.launch();
                    trace!("================================================================")
                }

                // (a, b, c, (d, e))
            });
            // ast.interact()
            //     .immediate_exclusive(|w| block_disposal.dispose(w));
        }
        Continue(())
    }
}

impl Check {
    // fn install_lints(&self, ctx: &mut Ctx) -> impl Disposable + use<> {
    //     let a = SingleModuleWithBrackets::install(ctx);
    //     let b = RLTLinkLint::install(ctx);
    //     let c = ShowLints::install(ctx);
    //     let d = DebugScopesLint::install(ctx);
    //     let e = DebugTypingLint::install(ctx);
    //
    //     ctx.immediate_exclusive(|w| {
    //         let mut lint_query = w.query::<&mut LintDescriptor>();
    //
    //         for mut descriptor in lint_query.iter_mut(w) {
    //             let found = self.enabled_lints.iter().any(|it| it == descriptor.name());
    //
    //             if found {
    //                 descriptor.enabled = true;
    //             }
    //         }
    //     });
    //
    //     (a, b, c, (d, e))
    // }

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
