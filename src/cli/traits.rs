use kodept::codespan_settings::Reports;
use kodept::common_iter::CommonIter;
use kodept::source_files::SourceView;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::ErrorReported;
use std::panic::{RefUnwindSafe, UnwindSafe};
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

pub trait Command {
    type Params: UnwindSafe;

    fn exec(
        &self,
        sources: impl CommonIter<Item = SourceView> + UnwindSafe,
        reports: &mut Reports,
        additional_params: Self::Params,
    ) -> Result<(), ErrorReported>
    where
        Self::Params: Clone + Send,
        Self: Sync + RefUnwindSafe,
    {
        let reports = reports.clone();
        match std::panic::catch_unwind(move || {
            sources.try_foreach_with((reports, additional_params), |(reports, params), source| {
                let path = source.path();
                let now = Instant::now();
                let result = reports.provide_collector(&source.clone(), |c| {
                    self.exec_for_source(source, c, params)
                });
                let (elapsed, suffix) = pick_appropriate_suffix(now.elapsed());
                warn!("Finished `{path}` in {:.2}{}", elapsed, suffix);
                result
            })
        }) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ErrorReported::new()),
        }
    }

    fn exec_for_source(
        &self,
        source: SourceView,
        collector: &mut ReportCollector,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported>;
}
