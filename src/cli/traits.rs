use kodept::report::{GlobalReports, Reports};
use kodept::source::collection::{SourceView, Sources};
use kodept_core::file_name::FileId;
use kodept_frontend::Execution;
use kodept_report::prelude::{ad_hoc_message, Diagnostic, Severity};
use std::ops::ControlFlow::{Break, Continue};
use std::panic::UnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
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

static PANICKED_SOURCE: OnceLock<FileId> = OnceLock::new();

struct SetPanickedSourceId(FileId);

impl Drop for SetPanickedSourceId {
    fn drop(&mut self) {
        if panicking() {
            _ = PANICKED_SOURCE.set(self.0);
        }
    }
}

pub trait CommandWithSources: Sized {
    fn build_sources(&self, report_collector: &GlobalReports) -> Execution<Sources>;

    fn exec(self, sources: Vec<SourceView>, reports: &Reports, output: PathBuf) -> Execution<()>
    where
        Self: UnwindSafe + Sync,
    {
        match std::panic::catch_unwind(move || {
            sources.into_iter().try_for_each(|source| {
                let _ = SetPanickedSourceId(*source.id);
                let now = Instant::now();
                let result = self.exec_for_source(source.clone(), reports, &output);
                let (elapsed, suffix) = pick_appropriate_suffix(now.elapsed());
                warn!("Finished `{}` in {elapsed:.2}{suffix}", source.path());
                result
            })
        }) {
            Ok(Continue(())) => Continue(()),
            Ok(Break(())) => Break(()),
            Err(_) => {
                reports.report(
                    *PANICKED_SOURCE.get().unwrap(),
                    ad_hoc_message(|| {
                        Diagnostic::new(Severity::Bug)
                            .with_message("Unknown panic happened. Contact Kodept developers.")
                    }),
                );
                Break(())
            }
        }
    }

    fn exec_for_source(
        &self,
        source: SourceView,
        reports: &Reports,
        output: &Path,
    ) -> Execution<()>;
}
