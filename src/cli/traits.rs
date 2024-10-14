use kodept::codespan_settings::{ProvideCollector, Reports};
use kodept::common_iter::CommonIter;
use kodept::source_files::{SourceFiles, SourceView};
use kodept_macros::error::report::Severity;
use kodept_macros::error::report_collector::{ReportCollector, Reporter};
use kodept_macros::error::Diagnostic;
use std::panic::UnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::thread::panicking;
use std::time::{Duration, Instant};
use tracing::warn;

fn pick_appropriate_suffix(dur: Duration) -> (f32, &'static str) {
    if dur < Duration::from_secs(1) {
        (dur.as_secs_f32() * 1000.0, "ms")
    } else if dur < Duration::from_secs(60) {
        (dur.as_secs_f32(), "s")
    } else if dur < Duration::from_secs(3600) {
        (dur.as_secs_f32() / 60.0, "min")
    } else {
        (dur.as_secs_f32() / 3600.0, "h")
    }
}

static PANICKED_SOURCE: AtomicU16 = AtomicU16::new(u16::MAX);

struct SetPanickedSourceId(u16);

impl Drop for SetPanickedSourceId {
    fn drop(&mut self) {
        if panicking() {
            PANICKED_SOURCE.store(self.0, Ordering::Relaxed);
        }
    }
}

pub trait CommandWithSources: Sized {
    fn build_sources(&self, collector: &mut ReportCollector<()>) -> Option<SourceFiles>;

    fn exec(self, sources: Arc<SourceFiles>, reports: &mut Reports, output: PathBuf) -> Option<()>
    where
        Self: UnwindSafe + Sync,
    {
        let rpt = reports.clone();
        let src = sources.clone();
        match std::panic::catch_unwind(move || {
            src.into_common_iter()
                .panic_fuse()
                .try_foreach_with(rpt, |reports, source| {
                    let _ = SetPanickedSourceId(*source.id);
                    let now = Instant::now();
                    let result = self.exec_for_source(source.clone(), reports, &output);
                    let (elapsed, suffix) = pick_appropriate_suffix(now.elapsed());
                    warn!("Finished `{}` in {elapsed:.2}{suffix}", source.path());
                    result
                })
        }) {
            Ok(Some(())) => Some(()),
            Ok(None) => None,
            Err(_) => {
                reports.provide_collector(&*sources, |c| {
                    c.report(
                        PANICKED_SOURCE.load(Ordering::Relaxed),
                        Diagnostic::new(Severity::Bug)
                            .with_message("Unknown panic happened. Contact Kodept developers."),
                    )
                });
                None
            }
        }
    }

    fn exec_for_source(
        &self,
        source: SourceView,
        reports: &mut Reports,
        output: &Path,
    ) -> Option<()>;
}
