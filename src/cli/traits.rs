use kodept::codespan_settings::CodespanSettings;
use kodept::common_iter::CommonIter;
use kodept::read_code_source::ReadCodeSource;
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
        sources: impl CommonIter<Item = ReadCodeSource> + UnwindSafe,
        settings: CodespanSettings,
        additional_params: Self::Params,
    ) -> Result<(), ErrorReported>
    where
        Self::Params: Clone + Send,
        Self: Sync + RefUnwindSafe,
    {
        match std::panic::catch_unwind(move || {
            sources.try_foreach_with(
                (settings, additional_params),
                |(settings, params), source| {
                    let path = source.path();
                    let now = Instant::now();
                    let result = self.exec_for_source(source, settings, params);
                    let (elapsed, suffix) = pick_appropriate_suffix(now.elapsed());
                    warn!("Finished `{path}` in {:.2}{}", elapsed, suffix);
                    result
                },
            )
        }) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ErrorReported::new()),
        }
    }

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported>;
}
