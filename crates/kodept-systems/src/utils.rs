use crate::source::collection::Reporter;
use bevy_ecs::prelude::*;
use kodept_core::try_port::Try;
use kodept_frontend::prelude::ExtractReports;
use kodept_report::prelude::IntoSpannedReportMessage;
use std::fmt::Debug;
use std::ops::ControlFlow;
use tracing::trace;

pub trait ReportSystemEx<In, Out, SystemMarker, ExtractMarker>
where
    In: SystemInput,
{
    fn extract_reports(self) -> impl IntoSystem<In, (), ()>;
}

pub trait LogSystemEx<In, Out, SystemMarker>
where
    In: SystemInput,
{
    fn trace_completion(self) -> impl IntoSystem<In, Out, ()>;

    fn trace_completion_with_name(self, name: &'static str) -> impl IntoSystem<In, Out, ()>;
}

impl<SystemMarker, ExtractMarker, Input, Out, T: IntoSystem<Input, Out, SystemMarker>>
    ReportSystemEx<Input, Out, SystemMarker, ExtractMarker> for T
where
    Out: Try<Output = ()> + 'static,
    Out::Residual: ExtractReports<ExtractMarker>,
    Input: SystemInput,
{
    #[track_caller]
    fn extract_reports(self) -> impl IntoSystem<Input, (), ()> {
        IntoSystem::into_system(self.pipe(|In(output): In<Out>, mut reporter: Reporter| {
            match output.branch() {
                ControlFlow::Continue(_) => {}
                ControlFlow::Break(e) => {
                    e.extract_reports(&mut reporter);
                }
            }
        }))
    }
}

impl<In, Out, SystemMarker, T> LogSystemEx<In, Out, SystemMarker> for T
where
    T: IntoSystem<In, Out, SystemMarker>,
    In: SystemInput,
    Out: Debug,
{
    #[track_caller]
    fn trace_completion(self) -> impl IntoSystem<In, Out, ()> {
        let name = std::any::type_name::<T>();
        self.trace_completion_with_name(name)
    }

    #[track_caller]
    fn trace_completion_with_name(self, name: &'static str) -> impl IntoSystem<In, Out, ()> {
        let id = self.system_type_id();
        let system = self.map(move |out| {
            trace!(system_id=?id, output=?out, "System `{name}` completed");
            out
        });
        IntoSystem::into_system(system)
    }
}

pub type ForwardReport<T> = Result<(), T>;

#[inline(always)]
pub fn forward<T>(report: T) -> ForwardReport<T>
where
    T: IntoSpannedReportMessage,
{
    Err(report)
}
