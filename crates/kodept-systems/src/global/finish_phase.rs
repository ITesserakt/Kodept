use crate::utils::{ForwardReport, ReportSystemEx, forward};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::resource::Resource;
use kodept_ecs::system::Res;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report_macros::Report;
use std::time::{Duration, Instant};

#[derive(Debug, Report)]
#[severity("note")]
#[message("Successfully completed in {:.3}{}", self.elapsed_value, self.elapsed_suffix)]
struct TotalTimeReport {
    elapsed_value: f64,
    elapsed_suffix: &'static str,
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

#[derive(Debug, Resource)]
struct TotalTime(Instant);

define_phase!(
    pub phase FinishPhase[FinishPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.instrumented = false;
        engine.insert_resource(TotalTime(Instant::now()));
        engine.add_systems(system.extract_reports())
    }
);

fn system(total_time: Res<TotalTime>) -> ForwardReport<TotalTimeReport> {
    let (duration, suffix) = pick_appropriate_suffix(total_time.0.elapsed());
    forward(TotalTimeReport {
        elapsed_value: duration,
        elapsed_suffix: suffix,
    })
}
