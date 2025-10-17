use bevy_ecs::prelude::*;
use kodept::utils::ReportSystemEx;
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use kodept_report::prelude::{Diagnostic, IntoSpannedReportMessage, Severity};
use std::time::{Duration, Instant};

struct TotalTimeReport(Duration);

impl IntoSpannedReportMessage for TotalTimeReport {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        let (duration, suffix) = pick_appropriate_suffix(self.0);

        Diagnostic::new(Severity::Note).with_message(format!(
            "Successfully completed in {:.3}{}",
            duration, suffix
        ))
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

#[derive(Debug, Resource)]
struct TotalTime(Instant);

define_phase!(
    pub phase FinishPhase[FinishPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.instrumented = false;
        engine.insert_resource(TotalTime(Instant::now()));
        engine.add_systems(system.report_errors())
    }
);

fn system(total_time: Res<TotalTime>) -> Result<(), TotalTimeReport> {
    Err(TotalTimeReport(total_time.0.elapsed()))
}
