use crate::source::collection::Reporter;
use kodept_core::try_port::Try;
use kodept_ecs::schedule::IntoScheduleConfigs;
use kodept_ecs::system::{In, IntoSystem, ScheduleSystem, SystemInput};
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

pub trait LogSystemSetEx<Marker> {
    fn trace_completion(self) -> impl IntoScheduleConfigs<ScheduleSystem, ()>;
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

macro_rules! impl_log_system_set_ex {
    ($([$t:ident, $m:ident, $field:tt]$(,)?)+) => {
        impl<$($t, $m, )+> LogSystemSetEx<($($m, )+)> for ($($t, )+)
        where
            $($t: IntoSystem<(), (), $m>,)+
        {
            fn trace_completion(self) -> impl IntoScheduleConfigs<ScheduleSystem, ()> {
                IntoScheduleConfigs::into_configs(
                    ($({self.$field}.trace_completion(),)+)
                )
            }
        }
    };
}

impl_log_system_set_ex! {
    [T1, M1, 0],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
    [T9, M9, 8],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
    [T9, M9, 8],
    [T10, M10, 9],
}
impl_log_system_set_ex! {
    [T1, M1, 0],
    [T2, M2, 1],
    [T3, M3, 2],
    [T4, M4, 3],
    [T5, M5, 4],
    [T6, M6, 5],
    [T7, M7, 6],
    [T8, M8, 7],
    [T9, M9, 8],
    [T10, M10, 9],
    [T11, M11, 10],
}

pub type ForwardReport<T> = Result<(), T>;

#[inline(always)]
pub fn forward<T>(report: T) -> ForwardReport<T>
where
    T: IntoSpannedReportMessage,
{
    Err(report)
}
